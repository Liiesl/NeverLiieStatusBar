use std::collections::HashMap;
use std::sync::OnceLock;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Globalization::{GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SLOCALIZEDDISPLAYNAME};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Input::Ime::{
    ImmGetConversionStatus, ImmGetContext, ImmGetDefaultIMEWnd, ImmReleaseContext,
    IME_CONVERSION_MODE, IME_SENTENCE_MODE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardLayoutList, HKL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, SendMessageW,
    WM_IME_CONTROL, WM_INPUTLANGCHANGEREQUEST,
};

const IMC_GETOPENSTATUS: usize = 0x0005;
const IME_CMODE_NATIVE: u32 = 0x0001;
const IME_CMODE_KATAKANA: u32 = 0x0002;

fn lang_id_map() -> &'static HashMap<u16, &'static str> {
    static MAP: OnceLock<HashMap<u16, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(0x0401, "AR"); m.insert(0x0402, "BG"); m.insert(0x0404, "ZH");
        m.insert(0x0405, "CS"); m.insert(0x0406, "DA"); m.insert(0x0407, "DE");
        m.insert(0x0408, "EL"); m.insert(0x0409, "EN"); m.insert(0x040A, "ES");
        m.insert(0x040B, "FI"); m.insert(0x040C, "FR"); m.insert(0x040D, "HE");
        m.insert(0x040E, "HU"); m.insert(0x040F, "IS"); m.insert(0x0410, "IT");
        m.insert(0x0411, "JA"); m.insert(0x0412, "KO"); m.insert(0x0413, "NL");
        m.insert(0x0414, "NO"); m.insert(0x0415, "PL"); m.insert(0x0416, "PT");
        m.insert(0x0418, "RO"); m.insert(0x0419, "RU"); m.insert(0x041A, "HR");
        m.insert(0x041B, "SK"); m.insert(0x041D, "SV"); m.insert(0x041E, "TH");
        m.insert(0x041F, "TR"); m.insert(0x0422, "UK"); m.insert(0x0425, "ET");
        m.insert(0x0426, "LV"); m.insert(0x0427, "LT"); m.insert(0x0804, "ZH");
        m.insert(0x0809, "EN"); m.insert(0x080A, "ES"); m.insert(0x080C, "FR");
        m.insert(0x0810, "IT"); m.insert(0x0813, "NL"); m.insert(0x0816, "PT");
        m.insert(0x081A, "SR"); m.insert(0x0C0A, "ES");
        m
    })
}

fn is_ime_openstatus_lang(lang_id: u16) -> bool {
    lang_id == 0x0412
}

fn non_ime_default(lang_id: u16) -> Option<&'static str> {
    match lang_id {
        0x0411 => Some("A"),
        0x0804 => Some("A"),
        0x0404 => Some("A"),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardLayout {
    pub hkl_raw: usize,
    pub short_name: String,
    pub display_name: String,
}

fn hkl_to_lang_id(hkl: &HKL) -> u16 {
    (hkl.0 as usize & 0xFFFF) as u16
}

fn get_short_lang_name(lang_id: u16) -> String {
    lang_id_map()
        .get(&lang_id)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("0x{:04X}", lang_id))
}

fn get_layout_display_name(hkl: &HKL) -> String {
    let lang_id = hkl_to_lang_id(hkl);

    unsafe {
        let mut locale_buf = [0u16; 85];
        LCIDToLocaleName(lang_id as u32, Some(&mut locale_buf), 0);

        let locale_name = String::from_utf16_lossy(
            &locale_buf[..locale_buf.iter().position(|&c| c == 0).unwrap_or(85)],
        );

        let mut name_buf = [0u16; 128];
        let size = GetLocaleInfoEx(
            &windows::core::HSTRING::from(&locale_name),
            LOCALE_SLOCALIZEDDISPLAYNAME,
            Some(&mut name_buf),
        );

        if size > 0 {
            return String::from_utf16_lossy(&name_buf[..(size as usize - 1)]);
        }
    }

    lang_id_map()
        .get(&lang_id)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Language 0x{:04X}", lang_id))
}

fn hwnd_belongs_to_our_process(hwnd: HWND) -> bool {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid == GetCurrentProcessId()
    }
}

pub fn get_real_foreground_hwnd() -> u64 {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return 0;
        }
        if hwnd_belongs_to_our_process(fg) {
            0
        } else {
            fg.0 as u64
        }
    }
}

pub fn get_active_layout_from(hwnd: HWND) -> Option<KeyboardLayout> {
    unsafe {
        if hwnd.0.is_null() {
            return None;
        }

        let thread_id = GetWindowThreadProcessId(hwnd, None);
        let hkl = GetKeyboardLayout(thread_id);
        let lang_id = hkl_to_lang_id(&hkl);

        Some(KeyboardLayout {
            hkl_raw: hkl.0 as usize,
            short_name: get_short_lang_name(lang_id),
            display_name: get_layout_display_name(&hkl),
        })
    }
}

pub fn get_active_layout() -> Option<KeyboardLayout> {
    let hwnd_raw = get_real_foreground_hwnd();
    if hwnd_raw == 0 {
        return None;
    }
    let hwnd = HWND(hwnd_raw as *mut core::ffi::c_void);
    get_active_layout_from(hwnd)
}

pub fn get_all_layouts() -> Vec<KeyboardLayout> {
    unsafe {
        let count = GetKeyboardLayoutList(None);
        if count <= 0 {
            return Vec::new();
        }

        let mut buffer = vec![HKL::default(); count as usize];
        let actual = GetKeyboardLayoutList(Some(&mut buffer));
        if actual <= 0 {
            return Vec::new();
        }

        buffer
            .iter()
            .take(actual as usize)
            .filter(|hkl| !hkl.0.is_null())
            .map(|hkl| {
                let lang_id = hkl_to_lang_id(hkl);
                KeyboardLayout {
                    hkl_raw: hkl.0 as usize,
                    short_name: get_short_lang_name(lang_id),
                    display_name: get_layout_display_name(hkl),
                }
            })
            .collect()
    }
}

pub fn switch_layout(target_hwnd: u64, hkl_raw: usize) {
    unsafe {
        let hwnd = HWND(target_hwnd as *mut core::ffi::c_void);
        let hkl = HKL(hkl_raw as *mut core::ffi::c_void);
        SendMessageW(
            hwnd,
            WM_INPUTLANGCHANGEREQUEST,
            Some(WPARAM(0)),
            Some(LPARAM(hkl.0 as isize)),
        );
    }
}

fn get_ime_indicator(lang_id: u16) -> Option<String> {
    unsafe {
        let fg_raw = get_real_foreground_hwnd();
        if fg_raw == 0 {
            return None;
        }
        let fg_hwnd = HWND(fg_raw as *mut core::ffi::c_void);

        let ime_wnd = ImmGetDefaultIMEWnd(fg_hwnd);
        if ime_wnd.0.is_null() {
            return None;
        }

        if is_ime_openstatus_lang(lang_id) {
            let is_open = SendMessageW(
                ime_wnd,
                WM_IME_CONTROL,
                Some(WPARAM(IMC_GETOPENSTATUS)),
                None,
            );
            if is_open.0 != 0 {
                return Some("가".to_string());
            }
            return None;
        }

        let himc = ImmGetContext(ime_wnd);
        if himc.0.is_null() {
            return None;
        }

        let mut conversion = IME_CONVERSION_MODE(0);
        let mut sentence = IME_SENTENCE_MODE(0);
        let ok = ImmGetConversionStatus(
            himc,
            Some(&mut conversion),
            Some(&mut sentence),
        );
        let _ = ImmReleaseContext(ime_wnd, himc);

        if ok.0 == 0 {
            return None;
        }

        let mode = conversion.0;
        match lang_id {
            0x0412 => {
                if mode & IME_CMODE_NATIVE != 0 {
                    return Some("가".to_string());
                }
            }
            0x0411 => {
                if mode & IME_CMODE_KATAKANA != 0 {
                    return Some("カ".to_string());
                }
                if mode & IME_CMODE_NATIVE != 0 {
                    return Some("あ".to_string());
                }
            }
            0x0804 | 0x0404 => {
                if mode & IME_CMODE_NATIVE != 0 {
                    return Some("中".to_string());
                }
            }
            _ => {}
        }

        non_ime_default(lang_id).map(|s| s.to_string())
    }
}

pub fn get_bar_text(hkl_raw: usize) -> String {
    if hkl_raw == 0 {
        return "??".to_string();
    }

    let lang_id = (hkl_raw & 0xFFFF) as u16;
    let short = get_short_lang_name(lang_id);

    if let Some(indicator) = get_ime_indicator(lang_id) {
        format!("{} {}", short, indicator)
    } else {
        short
    }
}
