use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use iced::widget::image::Handle as ImageHandle;

use crate::ipc::IconEventData;

// Windows system tray icon GUIDs to exclude (managed by Windows, not apps)
const EXCLUDED_GUIDS: &[&str] = &[
    "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // Volume / Bluetooth
    "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // Network
    "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // Battery
];

// Emergency cleanup: if process crashes, next instance can unhook the stale hook.
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SysTrayIconId {
    HandleUid(isize, u32),
    Guid(uuid::Uuid),
}

impl std::fmt::Display for SysTrayIconId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SysTrayIconId::HandleUid(h, u) => write!(f, "hwnd:{}:uid:{}", h, u),
            SysTrayIconId::Guid(g) => write!(f, "guid:{}", g),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemTrayIcon {
    pub id: SysTrayIconId,
    pub window_handle: Option<isize>,
    pub uid: Option<u32>,
    pub guid: Option<uuid::Uuid>,
    pub tooltip: String,
    pub icon_handle: Option<isize>,
    pub callback_message: Option<u32>,
    pub version: Option<u32>,
    pub icon_rgba: Option<Vec<u8>>,
    pub icon_width: u32,
    pub icon_height: u32,
    pub cached_image_handle: Option<ImageHandle>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum TrayIconAction {
    LeftClick,
    LeftDoubleClick,
    RightClick,
    MiddleClick,
}

// ============================================================================
// SystemTrayManager
// ============================================================================

pub struct SystemTrayManager {
    icons: HashMap<SysTrayIconId, SystemTrayIcon>,
    _hook_loader: Option<HookLoader>,
}

impl std::fmt::Debug for SystemTrayManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTrayManager")
            .field("icon_count", &self.icons.len())
            .finish()
    }
}

impl SystemTrayManager {
    pub fn new() -> Self {
        Self {
            icons: HashMap::new(),
            _hook_loader: Some(HookLoader::new()),
        }
    }

    pub fn icons(&self) -> &HashMap<SysTrayIconId, SystemTrayIcon> {
        &self.icons
    }

    fn find_icon_id(data: &IconEventData) -> Option<SysTrayIconId> {
        if let Some(guid) = data.guid {
            return Some(SysTrayIconId::Guid(guid));
        }
        if let (Some(handle), Some(uid)) = (data.window_handle, data.uid) {
            return Some(SysTrayIconId::HandleUid(handle, uid));
        }
        None
    }

    pub fn handle_event(&mut self, event: crate::ipc::Win32TrayEvent) {
        match event {
            crate::ipc::Win32TrayEvent::IconAdd { data }
            | crate::ipc::Win32TrayEvent::IconUpdate { data } => {
                self.handle_add_or_update(data);
            }
            crate::ipc::Win32TrayEvent::IconRemove { data } => {
                self.handle_remove(data);
            }
        }
    }

    fn handle_add_or_update(&mut self, data: IconEventData) {
        let icon_id = match Self::find_icon_id(&data) {
            Some(id) => id,
            None => return,
        };

        // Skip Windows system tray icons (volume, network, battery, etc.)
        if let SysTrayIconId::Guid(guid) = &icon_id
            && EXCLUDED_GUIDS.iter().any(|g| *g == guid.to_string()) {
                return;
            }

        // Deduplicate by HWND: if another icon with the same HWND exists (different UID),
        // remove the stale one. Windows/Explorer groups tray icons by process (same HWND),
        // so we only need to keep the most recently updated UID per HWND.
        if let (Some(hwnd), Some(_uid)) = (data.window_handle, data.uid) {
            let stale_ids: Vec<SysTrayIconId> = self
                .icons
                .iter()
                .filter(|(id, _icon)| {
                    matches!(id, SysTrayIconId::HandleUid(h, u) if *h == hwnd && *u != _uid)
                })
                .map(|(id, _)| id.clone())
                .collect();
            for id in stale_ids {
                self.icons.remove(&id);
            }
        }

        let (icon_rgba, icon_width, icon_height) = if let Some(handle) = data.icon_handle {
            if let Some((rgba, w, h)) = crate::icon_utils::hicon_to_rgba(handle) {
                (Some(rgba), w, h)
            } else {
                (None, 0, 0)
            }
        } else {
            (None, 0, 0)
        };

        let cached_image_handle = icon_rgba.as_ref().map(|rgba| {
            ImageHandle::from_rgba(icon_width, icon_height, rgba.clone())
        });

        if let Some(existing) = self.icons.get_mut(&icon_id) {
            if let Some(uid) = data.uid {
                existing.uid = Some(uid);
            }
            if let Some(handle) = data.window_handle {
                existing.window_handle = Some(handle);
            }
            if let Some(guid) = data.guid {
                existing.guid = Some(guid);
            }
            if let Some(tooltip) = &data.tooltip {
                existing.tooltip = tooltip.clone();
            }
            if icon_rgba.is_some() {
                existing.icon_handle = data.icon_handle;
                existing.icon_rgba = None;
                existing.icon_width = icon_width;
                existing.icon_height = icon_height;
                existing.cached_image_handle = cached_image_handle;
            }
            if let Some(cb) = data.callback_message {
                existing.callback_message = Some(cb);
            }
            if let Some(ver) = data.version {
                existing.version = Some(ver);
            }
        } else {
            let tooltip = data.tooltip.clone().unwrap_or_default();
            self.icons.insert(
                icon_id.clone(),
                SystemTrayIcon {
                    id: icon_id,
                    window_handle: data.window_handle,
                    uid: data.uid,
                    guid: data.guid,
                    tooltip,
                    icon_handle: data.icon_handle,
                    callback_message: data.callback_message,
                    version: data.version,
                    icon_rgba: None,
                    icon_width,
                    icon_height,
                    cached_image_handle,
                },
            );
        }
    }

    fn handle_remove(&mut self, data: IconEventData) {
        let icon_id = match Self::find_icon_id(&data) {
            Some(id) => id,
            None => return,
        };

        println!("[nl-tray] remove: id={}", icon_id);

        self.icons.remove(&icon_id);
    }

    pub fn send_action(&self, icon_id: &SysTrayIconId, action: &TrayIconAction) {
        let icon = match self.icons.get(icon_id) {
            Some(icon) => icon,
            None => return,
        };

        let window_handle = match icon.window_handle {
            Some(h) => h,
            None => return,
        };
        let uid = match icon.uid {
            Some(u) => u,
            None => return,
        };
        let callback = match icon.callback_message {
            Some(c) => c,
            None => return,
        };

        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                AllowSetForegroundWindow, GetWindowThreadProcessId,
                WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
                WM_RBUTTONDOWN, WM_RBUTTONUP,
            };

            let hwnd = HWND(window_handle as *mut _);

            let mut proc_id = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut proc_id));
            let _ = AllowSetForegroundWindow(proc_id);

            let wm_messages: Vec<u32> = match action {
                TrayIconAction::LeftClick => vec![WM_LBUTTONDOWN, WM_LBUTTONUP],
                TrayIconAction::LeftDoubleClick => vec![WM_LBUTTONDBLCLK, WM_LBUTTONUP],
                TrayIconAction::RightClick => vec![WM_RBUTTONDOWN, WM_RBUTTONUP],
                TrayIconAction::MiddleClick => vec![WM_MBUTTONDOWN, WM_MBUTTONUP],
            };

            for msg in wm_messages {
                send_notify_icon(hwnd, callback, uid, icon.version, msg);
            }

            if icon.version.is_some_and(|v| v >= 3) {
                if let TrayIconAction::RightClick = action {
                    send_notify_icon(
                        hwnd,
                        callback,
                        uid,
                        icon.version,
                        windows::Win32::UI::WindowsAndMessaging::WM_CONTEXTMENU,
                    );
                    return;
                }
                if let TrayIconAction::LeftClick = action {
                    send_notify_icon(hwnd, callback, uid, icon.version, windows::Win32::UI::Shell::NIN_SELECT);
                }
            }
        }
    }
}

unsafe fn send_notify_icon(
    hwnd: windows::Win32::Foundation::HWND,
    callback: u32,
    uid: u32,
    version: Option<u32>,
    message: u32,
) {
    use windows::Win32::Foundation::{LPARAM, POINT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SendNotifyMessageW};

    let wparam = if version.is_some_and(|v| v > 3) {
        let mut pt = POINT::default();
        unsafe { let _ = GetCursorPos(&mut pt); };
        pack_i32(pt.x as i16, pt.y as i16) as usize
    } else {
        uid as usize
    };

    let lparam = if version.is_some_and(|v| v > 3) {
        pack_i32(message as i16, uid as i16) as isize
    } else {
        pack_i32(message as i16, 0) as isize
    };

    unsafe { let _ = SendNotifyMessageW(hwnd, callback, WPARAM(wparam), LPARAM(lparam)); };
}

fn pack_i32(low: i16, high: i16) -> i32 {
    low as i32 | ((high as i32) << 16)
}

// ============================================================================
// Hook DLL Loader
// ============================================================================

type CallWndProcFn = unsafe extern "system" fn(
    i32,
    windows::Win32::Foundation::WPARAM,
    windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT;

pub struct HookLoader {
    #[allow(dead_code)]
    hook: Option<OwnedHook>,
    #[allow(dead_code)]
    dll: Option<OwnedDll>,
}

struct OwnedHook(windows::Win32::UI::WindowsAndMessaging::HHOOK);
#[allow(dead_code)]
struct OwnedDll(windows::Win32::Foundation::HMODULE);

impl Drop for OwnedHook {
    fn drop(&mut self) {
        HOOK_INSTALLED.store(false, Ordering::Relaxed);
        HOOK_HANDLE.store(0, Ordering::Relaxed);
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(self.0).ok();
        }
    }
}

impl Drop for OwnedDll {
    fn drop(&mut self) {
        // Hook is already uninstalled by OwnedHook drop before this (field drop order).
        // We intentionally do NOT call FreeLibrary here because:
        // 1. FreeLibrary during process shutdown can deadlock if DllMain is in flight
        // 2. The OS reclaims all memory/handles on process exit anyway
        // 3. Leaving the DLL loaded briefly after hook removal is harmless
    }
}

unsafe impl Send for HookLoader {}
unsafe impl Sync for HookLoader {}

impl HookLoader {
    pub fn new() -> Self {
        match Self::try_load() {
            Ok(loader) => {
                eprintln!("[nl-tray] Hook DLL loaded successfully");
                loader
            }
            Err(e) => {
                eprintln!("[nl-tray] Failed to load hook DLL: {}", e);
                Self { hook: None, dll: None }
            }
        }
    }

    fn try_load() -> Result<Self, String> {
        // Clean up stale hook from a previous crashed instance (same process, different run)
        if HOOK_INSTALLED.load(Ordering::Relaxed) {
            let old_handle = HOOK_HANDLE.load(Ordering::Relaxed);
            if old_handle != 0 {
                unsafe {
                    windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(
                        windows::Win32::UI::WindowsAndMessaging::HHOOK(old_handle as *mut _),
                    ).ok();
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        let dll_path = Self::get_dll_path()?;
        let dll_handle = Self::load_dll(&dll_path)?;

        let proc_addr = unsafe {
            Self::get_proc_address::<CallWndProcFn>(dll_handle, "CallWndProc")?
        };

        let shell_tray = Self::find_shell_tray()?;
        let thread_id = unsafe { Self::get_window_thread_id(shell_tray)? };

        let hook = unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetWindowsHookExW(
                windows::Win32::UI::WindowsAndMessaging::WH_CALLWNDPROC,
                Some(proc_addr),
                Some(dll_handle.into()),
                thread_id,
            )
        }
        .map_err(|e| format!("SetWindowsHookExW failed: {}", e))?;

        Self::refresh_icons();

        // Store handle for emergency cleanup if process crashes
        HOOK_INSTALLED.store(true, Ordering::Relaxed);
        HOOK_HANDLE.store(hook.0 as isize, Ordering::Relaxed);

        Ok(Self {
            hook: Some(OwnedHook(hook)),
            dll: Some(OwnedDll(dll_handle)),
        })
    }

    fn get_dll_path() -> Result<std::path::PathBuf, String> {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = exe_path.parent().ok_or("Failed to get exe directory")?;
        let dll_path = exe_dir.join("nl_tray_hook.dll");

        if !dll_path.exists() {
            let dev_path = exe_dir.join("tray-hook").join("nl_tray_hook.dll");
            if dev_path.exists() {
                return Ok(dev_path);
            }
            return Err(format!("DLL not found at: {}", dll_path.display()));
        }

        Ok(dll_path)
    }

    fn load_dll(path: &std::path::Path) -> Result<windows::Win32::Foundation::HMODULE, String> {
        let path_wide: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            windows::Win32::System::LibraryLoader::LoadLibraryW(
                windows::core::PCWSTR(path_wide.as_ptr()),
            )
            .map_err(|e| format!("LoadLibraryW failed: {}", e))
        }
    }

    unsafe fn get_proc_address<F>(dll: windows::Win32::Foundation::HMODULE, name: &str) -> Result<F, String> {
        let name_cstr = std::ffi::CString::new(name).map_err(|e| e.to_string())?;
        let addr = unsafe {
            windows::Win32::System::LibraryLoader::GetProcAddress(
                dll,
                windows::core::PCSTR(name_cstr.as_ptr() as _),
            )
        };
        match addr {
            Some(addr) => Ok(unsafe { std::mem::transmute_copy(&addr) }),
            None => Err(format!("GetProcAddress failed for: {}", name)),
        }
    }

    fn find_shell_tray() -> Result<isize, String> {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::FindWindowA;
            use windows_core::PCSTR;

            let class_name = std::ffi::CString::new("Shell_TrayWnd").unwrap();
            let pcstr = PCSTR::from_raw(class_name.as_ptr() as _);
            let hwnd = FindWindowA(pcstr, None)
                .map_err(|e| format!("FindWindowA failed: {}", e))?;
            Ok(hwnd.0 as isize)
        }
    }

    unsafe fn get_window_thread_id(hwnd: isize) -> Result<u32, String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

        let mut pid = 0u32;
        let tid = unsafe { GetWindowThreadProcessId(HWND(hwnd as *mut _), Some(&mut pid)) };
        if tid == 0 {
            return Err("GetWindowThreadProcessId failed".to_string());
        }
        Ok(tid)
    }

    fn refresh_icons() {
        unsafe {
            use windows::Win32::Foundation::{LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::{
                RegisterWindowMessageW, SendNotifyMessageW, HWND_BROADCAST,
            };
            use windows_core::w;

            let msg = RegisterWindowMessageW(w!("TaskbarCreated"));
            if msg != 0 {
                let _ = SendNotifyMessageW(HWND_BROADCAST, msg, WPARAM::default(), LPARAM::default());
            }
        }
    }
}
