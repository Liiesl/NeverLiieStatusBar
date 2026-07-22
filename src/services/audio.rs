use std::ffi::c_void;
use std::sync::{Mutex, mpsc};
use std::time::Duration;

use windows::core::{Interface, Result as WinResult, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, eConsole, eMultimedia, eRender, ERole,
    IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize,
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows_core::HSTRING;

// ---------------------------------------------------------------------------
// Volume change notification channel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEvent {
    VolumeChanged,
}

static AUDIO_EVENT_TX: Mutex<Option<mpsc::Sender<AudioEvent>>> = Mutex::new(None);

pub fn set_audio_event_sender(tx: mpsc::Sender<AudioEvent>) {
    *AUDIO_EVENT_TX.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
}

/// Create a receiver pair for audio events. Call this once at startup.
pub fn create_audio_event_channel() -> mpsc::Receiver<AudioEvent> {
    let (tx, rx) = mpsc::channel();
    set_audio_event_sender(tx);
    rx
}

/// Drain all pending audio events from the receiver. Called from MonitorTick.
#[allow(dead_code)]
pub fn drain_audio_events(rx: &mpsc::Receiver<AudioEvent>) -> bool {
    let mut had_events = false;
    while rx.try_recv().is_ok() {
        had_events = true;
    }
    had_events
}

// ---------------------------------------------------------------------------
// IAudioEndpointVolumeCallback implementation (manual vtable)
// ---------------------------------------------------------------------------

// We implement IAudioEndpointVolumeCallback manually to avoid version conflicts
// between windows-core 0.58.0 (from iced) and 0.62.2 (from this project).

#[repr(C)]
#[allow(non_snake_case)]
struct VolumeCallbackVtable {
    base__: windows::core::IUnknown_Vtbl,
    OnNotify: unsafe extern "system" fn(*mut c_void, *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> HRESULT,
}

unsafe extern "system" fn volume_callback_query_interface(
    this: *mut c_void,
    iid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        let target = GUID::from_u128(0x657804fa_d6ad_4496_8a60_352752af4f89);
        let iid_unknown = GUID::from_u128(0x00000000_0000_0000_C000_000000000046);
        if *iid == target || *iid == iid_unknown {
            *ppv = this;
            // vtable is *const *const (), index 1 = AddRef
            let vtable = *(this as *const *const *const ());
            let add_ref: unsafe extern "system" fn(*mut c_void) -> u32 =
                std::mem::transmute(vtable.add(1).read());
            let _ = add_ref(this);
            HRESULT(0)
        } else {
            *ppv = std::ptr::null_mut();
            HRESULT(0x80004002u32 as i32)
        }
    }
}

unsafe extern "system" fn volume_callback_add_ref(this: *mut c_void) -> u32 {
    unsafe {
        let callback = &*(this as *const VolumeCallbackInner);
        let count = callback.ref_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        count as u32
    }
}

unsafe extern "system" fn volume_callback_release(this: *mut c_void) -> u32 {
    unsafe {
        let callback = &*(this as *const VolumeCallbackInner);
        let count = callback.ref_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) - 1;
        if count == 0 {
            let _ = Box::from_raw(this as *mut VolumeCallbackInner);
        }
        count as u32
    }
}

unsafe extern "system" fn volume_on_notify(
    this: *mut c_void,
    pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA,
) -> HRESULT {
    unsafe {
        let callback = &*(this as *const VolumeCallbackInner);
        if let Some(data) = pnotify.as_ref() {
            let mut guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(s) = guard.as_mut() {
                if callback.is_speaker {
                    s.spk_volume = (data.fMasterVolume * 100.0).clamp(0.0, 100.0);
                    s.spk_muted = data.bMuted.as_bool();
                } else {
                    s.mic_volume = (data.fMasterVolume * 100.0).clamp(0.0, 100.0);
                    s.mic_muted = data.bMuted.as_bool();
                }
            }
        }
        if let Some(tx) = AUDIO_EVENT_TX.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let _ = tx.send(AudioEvent::VolumeChanged);
        }
        HRESULT(0) // S_OK
    }
}

static VOLUME_CALLBACK_VTABLE: VolumeCallbackVtable = VolumeCallbackVtable {
    base__: windows::core::IUnknown_Vtbl {
        QueryInterface: volume_callback_query_interface,
        AddRef: volume_callback_add_ref,
        Release: volume_callback_release,
    },
    OnNotify: volume_on_notify,
};

#[repr(C)]
struct VolumeCallbackInner {
    vtable: *const VolumeCallbackVtable,
    ref_count: std::sync::atomic::AtomicI32,
    is_speaker: bool,
}

impl VolumeCallbackInner {
    fn new(is_speaker: bool) -> Self {
        Self {
            vtable: &VOLUME_CALLBACK_VTABLE,
            ref_count: std::sync::atomic::AtomicI32::new(1),
            is_speaker,
        }
    }
}

struct VolumeCallbackPtr(*mut VolumeCallbackInner);

impl VolumeCallbackPtr {
    fn new(is_speaker: bool) -> Self {
        let inner = Box::new(VolumeCallbackInner::new(is_speaker));
        Self(Box::into_raw(inner))
    }
}

impl Drop for VolumeCallbackPtr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                volume_callback_release(self.0 as *mut c_void);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Audio callback registration
// ---------------------------------------------------------------------------

static VOLUME_CALLBACKS_REGISTERED: std::sync::Once = std::sync::Once::new();

/// Register volume callbacks by calling the COM vtable directly.
/// This bypasses the windows crate's Param trait to avoid the windows-core version conflict.
unsafe fn register_volume_callback_raw(
    volume: &IAudioEndpointVolume,
    is_speaker: bool,
) {
    let cb = VolumeCallbackPtr::new(is_speaker);
    let raw = cb.0 as *mut c_void;
    std::mem::forget(cb); // Leak to keep alive - COM will hold reference

    // Call the vtable function directly: RegisterControlChangeNotify(this, pnotify) -> HRESULT
    // IAudioEndpointVolume vtable: IUnknown(3) + RegisterControlChangeNotify = index 3
    let vtable = Interface::vtable(volume) as *const _ as *const *const ();
    unsafe {
        let fn_ptr = vtable.add(3).read();
        let register_fn: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT =
            std::mem::transmute(fn_ptr);
        let _ = register_fn(Interface::as_raw(volume) as *mut c_void, raw);
    }
}

fn register_volume_callbacks() {
    VOLUME_CALLBACKS_REGISTERED.call_once(|| {
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let need_uninit = hr.is_ok();

            // Register speaker callback
            if let Ok(dev_enum) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                if let Ok(device) = dev_enum.GetDefaultAudioEndpoint(eRender, eConsole) {
                    if let Ok(volume) = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                        register_volume_callback_raw(&volume, true);
                    }
                }
            }

            // Register mic callback
            if let Ok(dev_enum) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                if let Ok(device) = dev_enum.GetDefaultAudioEndpoint(eCapture, eConsole) {
                    if let Ok(volume) = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                        register_volume_callback_raw(&volume, false);
                    }
                }
            }

            if need_uninit {
                CoUninitialize();
            }
        }
    });
}

// ---------------------------------------------------------------------------
// IPolicyConfig (undocumented COM interface for setting default audio device)
// ---------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const POLICY_CONFIG_GUID: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

windows_core::imp::define_interface!(
    IPolicyConfig,
    IPolicyConfig_Vtbl,
    0xf8679f50_850a_41cf_9c72_430f290290c8
);

windows_core::imp::interface_hierarchy!(IPolicyConfig, windows_core::IUnknown);

#[allow(non_snake_case)]
impl IPolicyConfig {
    pub unsafe fn SetDefaultEndpoint(
        &self,
        device_id: impl windows::core::Param<PCWSTR>,
        role: ERole,
    ) -> WinResult<()> {
        unsafe {
            (Interface::vtable(self).SetDefaultEndpoint)(
                Interface::as_raw(self),
                device_id.param().abi(),
                role,
            )
            .ok()
        }
    }
}

#[repr(C)]
#[doc(hidden)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct IPolicyConfig_Vtbl {
    base__: ::windows::core::IUnknown_Vtbl,
    GetMixFormat: unsafe extern "system" fn(*mut c_void, PCWSTR, *mut *mut std::ffi::c_void) -> HRESULT,
    GetDeviceFormat: unsafe extern "system" fn(*mut c_void, PCWSTR, i32, *mut *mut std::ffi::c_void) -> HRESULT,
    ResetDeviceFormat: unsafe extern "system" fn(*mut c_void, PCWSTR) -> HRESULT,
    SetDeviceFormat: unsafe extern "system" fn(*mut c_void, PCWSTR, *mut std::ffi::c_void, *mut std::ffi::c_void) -> HRESULT,
    GetProcessingPeriod: unsafe extern "system" fn(*mut c_void, PCWSTR, i32, *mut i64, *mut i64) -> HRESULT,
    SetProcessingPeriod: unsafe extern "system" fn(*mut c_void, PCWSTR, *mut i64) -> HRESULT,
    GetShareMode: unsafe extern "system" fn(*mut c_void, PCWSTR, *mut std::ffi::c_void) -> HRESULT,
    SetShareMode: unsafe extern "system" fn(*mut c_void, PCWSTR, *mut std::ffi::c_void) -> HRESULT,
    GetPropertyValue: unsafe extern "system" fn(*mut c_void, PCWSTR, i32, *const PROPERTYKEY, *mut windows::Win32::System::Com::StructuredStorage::PROPVARIANT) -> HRESULT,
    SetPropertyValue: unsafe extern "system" fn(*mut c_void, PCWSTR, i32, *const PROPERTYKEY, *const windows::Win32::System::Com::StructuredStorage::PROPVARIANT) -> HRESULT,
    SetDefaultEndpoint: unsafe extern "system" fn(*mut c_void, PCWSTR, ERole) -> HRESULT,
    SetEndpointVisibility: unsafe extern "system" fn(*mut c_void, PCWSTR, i32) -> HRESULT,
}

// ---------------------------------------------------------------------------
// PKEY_Device_FriendlyName
// ---------------------------------------------------------------------------

/// {a45c254e-df1c-4efd-8020-67d146a850e0}; PID 14
const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
}

// ---------------------------------------------------------------------------
// COM helpers
// ---------------------------------------------------------------------------

fn com_context<F, T>(f: F) -> Option<T>
where
    F: FnOnce() -> Option<T>,
{
    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let need_uninit = hr.is_ok();
        let result = f();
        if need_uninit {
            CoUninitialize();
        }
        result
    }
}

fn get_spk_interface() -> Option<IAudioEndpointVolume> {
    com_context(|| unsafe {
        let dev_enum: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = dev_enum.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).ok()?;
        Some(volume)
    })
}

fn get_mic_interface() -> Option<IAudioEndpointVolume> {
    com_context(|| unsafe {
        let dev_enum: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = dev_enum.GetDefaultAudioEndpoint(eCapture, eConsole).ok()?;
        let volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).ok()?;
        Some(volume)
    })
}

#[allow(dead_code)]
fn pwstr_to_string(pwstr: &windows_core::PWSTR) -> String {
    if pwstr.0.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0usize;
        let mut ptr = pwstr.0;
        while *ptr != 0 {
            len += 1;
            ptr = ptr.add(1);
        }
        let slice = std::slice::from_raw_parts(pwstr.0, len);
        String::from_utf16_lossy(slice)
    }
}

fn get_default_device_id(dataflow: i32) -> Option<String> {
    com_context(|| unsafe {
        let dev_enum: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let dataflow_enum = if dataflow == 0 { eRender } else { eCapture };
        let device = dev_enum.GetDefaultAudioEndpoint(dataflow_enum, eConsole).ok()?;
        let id_pwstr = device.GetId().ok()?;
        Some(pwstr_to_string(&id_pwstr))
    })
}

fn device_id_to_string(device: &windows::Win32::Media::Audio::IMMDevice) -> Option<String> {
    unsafe {
        let pwstr = device.GetId().ok()?;
        Some(pwstr_to_string(&pwstr))
    }
}

fn get_device_friendly_name(device: &windows::Win32::Media::Audio::IMMDevice) -> Option<String> {
    unsafe {
        let store: IPropertyStore = device.OpenPropertyStore(STGM_READ).ok()?;
        let variant = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME).ok()?;
        let pwstr = variant.Anonymous.Anonymous.Anonymous.pwszVal;
        if pwstr.is_null() {
            return None;
        }
        let wide = pwstr.as_wide();
        Some(String::from_utf16_lossy(wide))
    }
}

// ---------------------------------------------------------------------------
// Device enumeration
// ---------------------------------------------------------------------------

pub fn scan_audio_devices() -> (Vec<AudioDevice>, Vec<AudioDevice>) {
    let mut out_devices = Vec::new();
    let mut in_devices = Vec::new();

    com_context(|| unsafe {
        let dev_enum: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;

        // Output (render) devices
        if let Ok(collection) = dev_enum.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
            let count = collection.GetCount().unwrap_or(0);
            for i in 0..count {
                if let Ok(device) = collection.Item(i) {
                    let id = device_id_to_string(&device).unwrap_or_default();
                    let name = get_device_friendly_name(&device)
                        .unwrap_or_else(|| "Unknown Device".to_string());
                    if !id.is_empty() {
                        out_devices.push(AudioDevice { name, id });
                    }
                }
            }
        }

        // Input (capture) devices
        if let Ok(collection) = dev_enum.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE) {
            let count = collection.GetCount().unwrap_or(0);
            for i in 0..count {
                if let Ok(device) = collection.Item(i) {
                    let id = device_id_to_string(&device).unwrap_or_default();
                    let name = get_device_friendly_name(&device)
                        .unwrap_or_else(|| "Unknown Device".to_string());
                    if !id.is_empty() {
                        in_devices.push(AudioDevice { name, id });
                    }
                }
            }
        }

        Some(())
    });

    out_devices.sort_by(|a, b| a.name.cmp(&b.name));
    in_devices.sort_by(|a, b| a.name.cmp(&b.name));

    (out_devices, in_devices)
}

// ---------------------------------------------------------------------------
// Device switching (IPolicyConfig)
// ---------------------------------------------------------------------------

pub fn set_default_device(device_id: &str, _is_input: bool) -> bool {
    com_context(|| unsafe {
        let dev_enum: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;

        let hdevice_id = HSTRING::from(device_id);
        let _device = dev_enum.GetDevice(PCWSTR(hdevice_id.as_ptr())).ok()?;

        let policy: IPolicyConfig = CoCreateInstance(&POLICY_CONFIG_GUID, None, CLSCTX_ALL).ok()?;

        let roles = [eConsole, eMultimedia, eCommunications];
        for role in roles {
            let _ = policy.SetDefaultEndpoint(PCWSTR(hdevice_id.as_ptr()), role);
        }

        Some(())
    })
    .is_some()
}

// ---------------------------------------------------------------------------
// Get current default device ID
// ---------------------------------------------------------------------------

pub fn get_current_output_device_id() -> Option<String> {
    get_default_device_id(0)
}

pub fn get_current_input_device_id() -> Option<String> {
    get_default_device_id(1)
}

// ---------------------------------------------------------------------------
// Volume / Mute (existing API)
// ---------------------------------------------------------------------------

static AUDIO_STATE: Mutex<Option<AudioState>> = Mutex::new(None);

struct AudioState {
    spk_volume: f32,
    mic_volume: f32,
    spk_muted: bool,
    mic_muted: bool,
}

fn ensure_init() {
    let mut guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_some() {
        return;
    }

    let mut state = AudioState {
        spk_volume: 0.0,
        mic_volume: 0.0,
        spk_muted: false,
        mic_muted: false,
    };

    if let Some(spk) = get_spk_interface() {
        unsafe {
            if let Ok(v) = spk.GetMasterVolumeLevelScalar() {
                state.spk_volume = (v * 100.0).clamp(0.0, 100.0);
            }
            if let Ok(m) = spk.GetMute() {
                state.spk_muted = m.as_bool();
            }
        }
    }

    if let Some(mic) = get_mic_interface() {
        unsafe {
            if let Ok(v) = mic.GetMasterVolumeLevelScalar() {
                state.mic_volume = (v * 100.0).clamp(0.0, 100.0);
            }
            if let Ok(m) = mic.GetMute() {
                state.mic_muted = m.as_bool();
            }
        }
    }

    *guard = Some(state);

    // Register volume change callbacks (once globally)
    drop(guard);
    register_volume_callbacks();
}

pub fn get_speaker_volume() -> f32 {
    ensure_init();
    let guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().map_or(0.0, |s| s.spk_volume)
}

pub fn get_mic_volume() -> f32 {
    ensure_init();
    let guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().map_or(0.0, |s| s.mic_volume)
}

pub fn is_speaker_muted() -> bool {
    ensure_init();
    let guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().is_some_and(|s| s.spk_muted)
}

pub fn is_mic_muted() -> bool {
    ensure_init();
    let guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().is_some_and(|s| s.mic_muted)
}

pub fn set_speaker_volume(value: f32) {
    let scalar = (value / 100.0).clamp(0.0, 1.0);
    if let Some(iface) = get_spk_interface() {
        unsafe {
            let _ = iface.SetMasterVolumeLevelScalar(scalar, std::ptr::null());
            if value > 0.0 {
                let _ = iface.SetMute(false, std::ptr::null());
            }
        }
    }
    let mut guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = guard.as_mut() {
        s.spk_volume = value.clamp(0.0, 100.0);
        if value > 0.0 {
            s.spk_muted = false;
        }
    }
}

pub fn set_mic_volume(value: f32) {
    let scalar = (value / 100.0).clamp(0.0, 1.0);
    if let Some(iface) = get_mic_interface() {
        unsafe {
            let _ = iface.SetMasterVolumeLevelScalar(scalar, std::ptr::null());
        }
    }
    let mut guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = guard.as_mut() {
        s.mic_volume = value.clamp(0.0, 100.0);
    }
}

pub fn toggle_speaker_mute() {
    if let Some(iface) = get_spk_interface() {
        unsafe {
            if let Ok(muted) = iface.GetMute() {
                let _ = iface.SetMute(!muted.as_bool(), std::ptr::null());
            }
        }
    }
    sync_speaker_state();
}

pub fn toggle_mic_mute() {
    if let Some(iface) = get_mic_interface() {
        unsafe {
            if let Ok(muted) = iface.GetMute() {
                let _ = iface.SetMute(!muted.as_bool(), std::ptr::null());
            }
        }
    }
    sync_mic_state();
}

fn sync_speaker_state() {
    if let Some(iface) = get_spk_interface() {
        unsafe {
            let mut guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(s) = guard.as_mut() {
                if let Ok(v) = iface.GetMasterVolumeLevelScalar() {
                    s.spk_volume = (v * 100.0).clamp(0.0, 100.0);
                }
                if let Ok(m) = iface.GetMute() {
                    s.spk_muted = m.as_bool();
                }
            }
        }
    }
}

fn sync_mic_state() {
    if let Some(iface) = get_mic_interface() {
        unsafe {
            let mut guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(s) = guard.as_mut() {
                if let Ok(v) = iface.GetMasterVolumeLevelScalar() {
                    s.mic_volume = (v * 100.0).clamp(0.0, 100.0);
                }
                if let Ok(m) = iface.GetMute() {
                    s.mic_muted = m.as_bool();
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn sync_all() {
    ensure_init();
    sync_speaker_state();
    sync_mic_state();
}

// ---------------------------------------------------------------------------
// Media Player (WinRT SMTC)
// ---------------------------------------------------------------------------

use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

#[derive(Debug, Clone)]
pub struct MediaPlayerState {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub thumbnail: Vec<u8>,
    pub is_playing: bool,
}

fn smtc_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

fn block_on_smtc<F: std::future::Future>(f: F) -> F::Output {
    smtc_runtime().block_on(f)
}

async fn get_smtc_manager() -> Option<GlobalSystemMediaTransportControlsSessionManager> {
    let op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().ok()?;
    op.await.ok()
}

async fn fetch_thumbnail_bytes(
    thumb_ref: &windows::Storage::Streams::IRandomAccessStreamReference,
) -> Vec<u8> {
    let stream_op = match thumb_ref.OpenReadAsync() {
        Ok(op) => op,
        Err(_) => return Vec::new(),
    };
    let stream = match stream_op.await {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let size = match stream.Size() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    if size == 0 {
        return Vec::new();
    }
    let input = match stream.GetInputStreamAt(0) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let reader = match windows::Storage::Streams::DataReader::CreateDataReader(&input) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let sz = size as u32;
    let load_op = match reader.LoadAsync(sz) {
        Ok(op) => op,
        Err(_) => return Vec::new(),
    };
    match load_op.await {
        Ok(loaded) => {
            if loaded != sz {
                return Vec::new();
            }
        }
        Err(_) => return Vec::new(),
    }
    let mut buf = vec![0u8; sz as usize];
    reader.ReadBytes(&mut buf).ok();
    buf
}

async fn get_player_state(session: &GlobalSystemMediaTransportControlsSession) -> Option<MediaPlayerState> {
    let id = session.SourceAppUserModelId().ok()?.to_string();

    let is_playing = session.GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false);

    let props = session.TryGetMediaPropertiesAsync().ok()?.await.ok()?;

    let title = props.Title().ok().map(|t| t.to_string()).unwrap_or_default();
    let artist = props.Artist().ok().map(|a| a.to_string()).unwrap_or_default();

    let thumbnail = if let Ok(thumb_ref) = props.Thumbnail() {
        fetch_thumbnail_bytes(&thumb_ref).await
    } else {
        Vec::new()
    };

    Some(MediaPlayerState {
        id,
        title,
        artist,
        thumbnail,
        is_playing,
    })
}

async fn get_all_media_players_inner() -> Vec<MediaPlayerState> {
    let mgr = match get_smtc_manager().await {
        Some(m) => m,
        None => return Vec::new(),
    };

    let sessions = match mgr.GetSessions() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut players = Vec::new();
    for session in sessions.into_iter() {
        if let Some(state) = get_player_state(&session).await {
            players.push(state);
        }
    }
    players
}

pub fn get_all_media_players_sync() -> Vec<MediaPlayerState> {
    block_on_smtc(get_all_media_players_inner())
}

pub fn media_toggle_play_sync(player_id: &str) {
    block_on_smtc(async {
        let mgr = match get_smtc_manager().await {
            Some(m) => m,
            None => return,
        };
        let sessions = match mgr.GetSessions() {
            Ok(s) => s,
            Err(_) => return,
        };
        for session in sessions.into_iter() {
            if let Ok(id) = session.SourceAppUserModelId() {
                if id.to_string() == player_id {
                    if let Ok(op) = session.TryTogglePlayPauseAsync() {
                        let _ = op.await;
                    }
                    return;
                }
            }
        }
    });
}

pub fn media_next_track_sync(player_id: &str) {
    block_on_smtc(async {
        let mgr = match get_smtc_manager().await {
            Some(m) => m,
            None => return,
        };
        let sessions = match mgr.GetSessions() {
            Ok(s) => s,
            Err(_) => return,
        };
        for session in sessions.into_iter() {
            if let Ok(id) = session.SourceAppUserModelId() {
                if id.to_string() == player_id {
                    if let Ok(op) = session.TrySkipNextAsync() {
                        let _ = op.await;
                    }
                    return;
                }
            }
        }
    });
}

pub fn media_prev_track_sync(player_id: &str) {
    block_on_smtc(async {
        let mgr = match get_smtc_manager().await {
            Some(m) => m,
            None => return,
        };
        let sessions = match mgr.GetSessions() {
            Ok(s) => s,
            Err(_) => return,
        };
        for session in sessions.into_iter() {
            if let Ok(id) = session.SourceAppUserModelId() {
                if id.to_string() == player_id {
                    if let Ok(op) = session.TrySkipPreviousAsync() {
                        let _ = op.await;
                    }
                    return;
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// SMTC Event-Driven Updates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaEvent {
    /// Sessions changed or updated — re-enumerate all players
    Changed,
}

static MEDIA_EVENT_TX: Mutex<Option<mpsc::Sender<MediaEvent>>> = Mutex::new(None);

pub fn create_media_event_channel() -> mpsc::Receiver<MediaEvent> {
    let (tx, rx) = mpsc::channel();
    *MEDIA_EVENT_TX.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    rx
}

fn send_media_event(event: MediaEvent) {
    if let Some(tx) = MEDIA_EVENT_TX.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        let _ = tx.send(event);
    }
}

// We store the SMTC manager in a static so the SessionsChanged callback can
// access it to enumerate sessions. This is safe because the manager is
// reference-counted internally and never drops while events are registered.
static SMTC_MANAGER: Mutex<Option<GlobalSystemMediaTransportControlsSessionManager>> =
    Mutex::new(None);

/// Subscribe to per-session events (properties + playback) for a single session.
/// Returns Ok(()) on success. Safe to call multiple times for the same session —
/// the OS deduplicates, but we track IDs to avoid redundancy.
unsafe fn subscribe_to_session(
    session: &GlobalSystemMediaTransportControlsSession,
    known_ids: &mut std::collections::HashSet<String>,
) {
    use windows::Foundation::TypedEventHandler;

    let id = match session.SourceAppUserModelId() {
        Ok(id) => id.to_string(),
        Err(_) => return,
    };

    if known_ids.contains(&id) {
        return;
    }

    let _ = session.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
        send_media_event(MediaEvent::Changed);
        Ok(())
    }));

    let _ = session.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
        send_media_event(MediaEvent::Changed);
        Ok(())
    }));

    known_ids.insert(id);
}

/// Called by the SessionsChanged callback. Diffs known sessions against current
/// sessions and subscribes to any new ones.
fn on_sessions_changed() {
    let mgr = match SMTC_MANAGER.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(m) => m.clone(),
            None => {
                return;
            }
        },
        Err(_) => return,
    };

    let sessions = match mgr.GetSessions() {
        Ok(s) => s,
        Err(_) => {
            return;
        }
    };

    let mut known_ids = std::collections::HashSet::new();
    for session in sessions.into_iter() {
        unsafe { subscribe_to_session(&session, &mut known_ids) };
    }

    send_media_event(MediaEvent::Changed);
}

/// Start the SMTC event listener on a dedicated thread.
/// Subscribes to manager SessionsChanged ONCE. The SessionsChanged callback
/// itself detects new sessions and subscribes to their events.
pub fn start_smtc_listener() {
    std::thread::spawn(|| {
        // Initialize COM on this thread — all WinRT calls happen here
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let _ = hr;
        }

        // Get the SMTC manager once
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(r) => r,
            Err(_) => {
                return;
            }
        };

        let mgr = rt.block_on(async {
            let op = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                Ok(op) => op,
                Err(_) => {
                    return None;
                }
            };
            match op.await {
                Ok(m) => Some(m),
                Err(_) => {
                    None
                }
            }
        });

        let mgr = match mgr {
            Some(m) => m,
            None => {
                return;
            }
        };

        // Store in static so the callback can access it
        if let Ok(mut guard) = SMTC_MANAGER.lock() {
            *guard = Some(mgr.clone());
        }

        // Subscribe to SessionsChanged ONCE
        use windows::Foundation::TypedEventHandler;
        let _ = mgr.SessionsChanged(&TypedEventHandler::new(|_, _| {
            on_sessions_changed();
            Ok(())
        }));

        // Do initial session enumeration + subscription
        on_sessions_changed();

        // Keep the thread alive. The COM apartment must stay alive for
        // WinRT event callbacks to keep firing.
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }

        // unreachable but needed for type inference
        #[allow(unreachable_code)]
        {
            drop(rt);
        }
    });
}
