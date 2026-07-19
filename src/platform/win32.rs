#[cfg(target_os = "windows")]
mod inner {
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::Graphics::Dwm::{DwmExtendFrameIntoClientArea, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
    use windows::Win32::UI::Controls::MARGINS;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowLongPtrW, SetWindowLongPtrW,
        SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };

    const SYSTEM_CLASSES: &[&str] = &[
        "Progman",
        "WorkerW",
        "Shell_TrayWnd",
        "Shell_SecondaryTrayWnd",
    ];

    pub fn is_foreground_blocked(our_hwnd: u64) -> bool {
        unsafe {
            let fg = GetForegroundWindow();
            if fg.0.is_null() {
                return false;
            }
            if fg.0 as u64 == our_hwnd {
                return true;
            }

            let mut buf = [0u16; 256];
            let len = GetClassNameW(fg, &mut buf);
            if len == 0 {
                return true;
            }

            let name = String::from_utf16_lossy(&buf[..len as usize]);
            !SYSTEM_CLASSES.iter().any(|c| *c == name)
        }
    }

    pub fn apply_window_flags(hwnd: u64) {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_style = (ex_style
                | WS_EX_TOOLWINDOW.0 as isize
                | WS_EX_NOACTIVATE.0 as isize)
                & !WS_EX_APPWINDOW.0 as isize;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
        }
    }

    pub fn apply_popup_flags(hwnd: u64) {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_style = (ex_style
                | WS_EX_TOOLWINDOW.0 as isize
                | WS_EX_NOACTIVATE.0 as isize)
                & !WS_EX_APPWINDOW.0 as isize;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
        }
    }

    pub fn apply_dwm_rounded_corners(hwnd: u64) {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);
            let margins = MARGINS {
                cxLeftWidth: -1,
                cxRightWidth: -1,
                cyTopHeight: -1,
                cyBottomHeight: -1,
            };
            let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

            let dark_mode: i32 = 1;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &dark_mode as *const i32 as *const _,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }

    pub fn force_z_order(hwnd: u64) {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    pub fn get_cursor_pos() -> Option<(f32, f32)> {
        unsafe {
            let mut pt = POINT::default();
            if GetCursorPos(&mut pt).is_ok() {
                Some((pt.x as f32, pt.y as f32))
            } else {
                None
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod inner {
    pub fn is_foreground_blocked(_our_hwnd: u64) -> bool {
        false
    }
    pub fn apply_window_flags(_hwnd: u64) {}
    pub fn apply_popup_flags(_hwnd: u64) {}
    pub fn apply_dwm_rounded_corners(_hwnd: u64) {}
    pub fn force_z_order(_hwnd: u64) {}
    pub fn get_cursor_pos() -> Option<(f32, f32)> {
        None
    }
}

pub use inner::{apply_dwm_rounded_corners, apply_popup_flags, apply_window_flags, force_z_order, get_cursor_pos, is_foreground_blocked};
