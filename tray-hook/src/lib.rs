use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM, LRESULT, WPARAM},
    System::DataExchange::COPYDATASTRUCT,
    UI::{
        Shell::{
            NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
            NIM_SETVERSION, NIS_HIDDEN, NOTIFYICONDATAW_0, NOTIFY_ICON_DATA_FLAGS,
            NOTIFY_ICON_INFOTIP_FLAGS, NOTIFY_ICON_MESSAGE, NOTIFY_ICON_STATE,
        },
        WindowsAndMessaging::{CallNextHookEx, GetClassNameW, CWPSTRUCT, WM_COPYDATA},
    },
};
use windows_core::GUID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconEventData {
    pub uid: Option<u32>,
    pub window_handle: Option<isize>,
    pub guid: Option<uuid::Uuid>,
    pub tooltip: Option<String>,
    pub icon_handle: Option<isize>,
    pub callback_message: Option<u32>,
    pub version: Option<u32>,
    pub is_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Win32TrayEvent {
    IconAdd { data: IconEventData },
    IconUpdate { data: IconEventData },
    IconRemove { data: IconEventData },
}

#[repr(C)]
struct ShellTrayMessage {
    magic_number: i32,
    message_type: u32,
    icon_data: NotifyIconData,
    version: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NotifyIconData {
    callback_size: u32,
    window_handle: u32,
    uid: u32,
    flags: NOTIFY_ICON_DATA_FLAGS,
    callback_message: u32,
    icon_handle: u32,
    tooltip: [u16; 128],
    state: NOTIFY_ICON_STATE,
    state_mask: NOTIFY_ICON_STATE,
    size_info: [u16; 256],
    anonymous: NOTIFYICONDATAW_0,
    info_title: [u16; 64],
    info_flags: NOTIFY_ICON_INFOTIP_FLAGS,
    guid_item: GUID,
    balloon_icon_handle: u32,
}

impl From<NotifyIconData> for IconEventData {
    fn from(icon_data: NotifyIconData) -> Self {
        let icon_handle = if icon_data.icon_handle != 0 && icon_data.flags.0 & NIF_ICON.0 != 0 {
            Some(icon_data.icon_handle as isize)
        } else {
            None
        };

        let guid =
            if icon_data.guid_item != GUID::default() && icon_data.flags.0 & NIF_GUID.0 != 0 {
                Some(uuid::Uuid::from_u128(icon_data.guid_item.to_u128()))
            } else {
                None
            };

        let tooltip = if icon_data.flags.0 & NIF_TIP.0 != 0 {
            let tooltip_len = icon_data.tooltip.iter().position(|&c| c == 0).unwrap_or(0);
            let tooltip_str = String::from_utf16_lossy(&icon_data.tooltip[..tooltip_len])
                .replace('\r', "")
                .to_string();
            (!tooltip_str.is_empty()).then_some(tooltip_str)
        } else {
            None
        };

        let (window_handle, uid) = if icon_data.window_handle != 0 {
            (Some(icon_data.window_handle as isize), Some(icon_data.uid))
        } else {
            (None, None)
        };

        let callback_message = if icon_data.flags.contains(NIF_MESSAGE) {
            Some(icon_data.callback_message)
        } else {
            None
        };

        let version = if unsafe { icon_data.anonymous.uVersion } > 0
            && unsafe { icon_data.anonymous.uVersion } <= 4
        {
            Some(unsafe { icon_data.anonymous.uVersion })
        } else {
            None
        };

        let is_visible = icon_data.state.0 & NIS_HIDDEN.0 == 0;

        IconEventData {
            uid,
            window_handle,
            guid,
            tooltip,
            icon_handle,
            callback_message,
            version,
            is_visible,
        }
    }
}

fn get_window_class(hwnd: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetClassNameW(hwnd, &mut text) };
    let length = usize::try_from(len).unwrap_or(0);
    String::from_utf16_lossy(&text[..length])
}

fn pipe_name() -> String {
    let session_id = unsafe {
        let mut sid = 0u32;
        let _ = windows::Win32::System::RemoteDesktop::ProcessIdToSessionId(
            std::process::id(),
            &mut sid,
        );
        sid
    };
    format!(r"\\.\pipe\nl-tray-{}", session_id)
}

fn send_event_via_ipc(event: Win32TrayEvent) {
    let Ok(data) = serde_json::to_vec(&event) else {
        return;
    };

    let pipe_name = pipe_name();
    let pipe_name_wide: Vec<u16> = pipe_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, OPEN_EXISTING, ReadFile, WriteFile,
            FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_MODE,
        };

        let handle = CreateFileW(
            windows::core::PCWSTR(pipe_name_wide.as_ptr()),
            FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        );

        let Ok(handle) = handle else {
            return;
        };

        let mut bytes_written = 0u32;
        let _ = WriteFile(
            handle,
            Some(data.as_slice()),
            Some(&mut bytes_written),
            None,
        );

        let mut response = [0u8; 64];
        let mut bytes_read = 0u32;
        let _ = ReadFile(
            handle,
            Some(&mut response),
            Some(&mut bytes_read),
            None,
        );

        let _ = CloseHandle(handle);
    }
}

/// WH_CALLWNDPROC hook procedure.
///
/// # Safety
#[no_mangle]
pub unsafe extern "system" fn CallWndProc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let next = || CallNextHookEx(None, code, wparam, lparam);
    if code < 0 {
        return next();
    }

    let Some(msg) = (lparam.0 as *const CWPSTRUCT).as_ref() else {
        return next();
    };

    let class = get_window_class(msg.hwnd);
    if class != "Shell_TrayWnd" {
        return next();
    }

    if let Some(event) = process_tray_message(msg) {
        send_event_via_ipc(event);
    }

    next()
}

unsafe fn process_tray_message(msg: &CWPSTRUCT) -> Option<Win32TrayEvent> {
    if msg.message != WM_COPYDATA {
        return None;
    }

    let copy_data = (msg.lParam.0 as *const COPYDATASTRUCT).as_ref()?;

    if copy_data.dwData != 1 || copy_data.lpData.is_null() {
        return None;
    }

    let tray_message = &*copy_data.lpData.cast::<ShellTrayMessage>();
    let icon_data: IconEventData = tray_message.icon_data.into();

    match NOTIFY_ICON_MESSAGE(tray_message.message_type) {
        NIM_ADD => Some(Win32TrayEvent::IconAdd { data: icon_data }),
        NIM_MODIFY | NIM_SETVERSION => Some(Win32TrayEvent::IconUpdate { data: icon_data }),
        NIM_DELETE => Some(Win32TrayEvent::IconRemove { data: icon_data }),
        _ => None,
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _hinst_dll: windows::Win32::Foundation::HINSTANCE,
    _fdw_reason: u32,
    _lpv_reserved: *const std::ffi::c_void,
) -> bool {
    true
}
