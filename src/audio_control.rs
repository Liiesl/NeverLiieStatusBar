use std::ffi::c_void;
use std::sync::Mutex;

use windows::core::{Interface, Result as WinResult, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, eConsole, eMultimedia, eRender, ERole,
    IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize,
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows_core::HSTRING;

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

#[derive(Debug, Clone, Default)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub thumbnail: Vec<u8>,
    pub is_playing: bool,
    pub has_session: bool,
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
    guard.as_ref().map_or(false, |s| s.spk_muted)
}

pub fn is_mic_muted() -> bool {
    ensure_init();
    let guard = AUDIO_STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().map_or(false, |s| s.mic_muted)
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

pub fn sync_all() {
    ensure_init();
    sync_speaker_state();
    sync_mic_state();
}

// ---------------------------------------------------------------------------
// Media Player (WinRT SMTC)
// ---------------------------------------------------------------------------

use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

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

async fn get_media_state_inner() -> MediaState {
    let mgr = match get_smtc_manager().await {
        Some(m) => m,
        None => return MediaState::default(),
    };

    let session = match mgr.GetCurrentSession() {
        Ok(s) => s,
        Err(_) => return MediaState::default(),
    };

    let mut state = MediaState {
        has_session: true,
        ..Default::default()
    };

    if let Ok(info) = session.GetPlaybackInfo() {
        if let Ok(status) = info.PlaybackStatus() {
            state.is_playing =
                status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
        }
    }

    if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
        if let Ok(props) = props_op.await {
            state.title = props
                .Title()
                .ok()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "Unknown Title".to_string());
            state.artist = props
                .Artist()
                .ok()
                .map(|a| a.to_string())
                .unwrap_or_default();

            if let Ok(thumb_ref) = props.Thumbnail() {
                state.thumbnail = fetch_thumbnail_bytes(&thumb_ref).await;
            }
        }
    }

    state
}

/// Synchronous wrapper - only call from spawn_blocking or non-tokio threads
pub fn get_media_state_sync() -> MediaState {
    block_on_smtc(get_media_state_inner())
}

pub fn media_toggle_play_sync() {
    block_on_smtc(async {
        let mgr = match get_smtc_manager().await {
            Some(m) => m,
            None => return,
        };
        if let Ok(session) = mgr.GetCurrentSession() {
            if let Ok(op) = session.TryTogglePlayPauseAsync() {
                let _ = op.await;
            }
        }
    });
}

pub fn media_next_track_sync() {
    block_on_smtc(async {
        let mgr = match get_smtc_manager().await {
            Some(m) => m,
            None => return,
        };
        if let Ok(session) = mgr.GetCurrentSession() {
            if let Ok(op) = session.TrySkipNextAsync() {
                let _ = op.await;
            }
        }
    });
}

pub fn media_prev_track_sync() {
    block_on_smtc(async {
        let mgr = match get_smtc_manager().await {
            Some(m) => m,
            None => return,
        };
        if let Ok(session) = mgr.GetCurrentSession() {
            if let Ok(op) = session.TrySkipPreviousAsync() {
                let _ = op.await;
            }
        }
    });
}
