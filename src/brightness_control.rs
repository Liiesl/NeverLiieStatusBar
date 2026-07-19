use std::sync::Mutex;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Wmi::{
    WbemLocator, IWbemLocator, IWbemServices, IWbemClassObject,
    WBEM_GENERIC_FLAG_TYPE,
};
use windows::Win32::System::Variant::VARIANT;
use windows_core::BSTR;

static BRIGHTNESS_STATE: Mutex<Option<f32>> = Mutex::new(None);

unsafe fn wmi_connect() -> Option<IWbemServices> {
    let locator: IWbemLocator = unsafe {
        CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).ok()?
    };
    let empty = BSTR::new();
    let root = BSTR::from("root\\WMI");
    unsafe {
        locator.ConnectServer(
            &root, &empty, &empty, &empty, 0, &empty, None,
        ).ok()
    }
}

unsafe fn wmi_get_brightness_inner() -> Option<f32> {
    let services = unsafe { wmi_connect() }?;
    let lang = BSTR::from("WQL");
    let query = BSTR::from("SELECT * FROM WmiMonitorBrightness");
    let flags = WBEM_GENERIC_FLAG_TYPE(48);

    let enumerator = unsafe { services.ExecQuery(&lang, &query, flags, None).ok()? };

    let mut obj: Option<IWbemClassObject> = None;
    let mut returned = 0u32;
    unsafe {
        if enumerator.Next(-1, std::slice::from_mut(&mut obj), &mut returned).is_err() {
            return None;
        }
    }

    let obj = obj?;
    let mut variant = VARIANT::default();
    let prop = BSTR::from("CurrentBrightness");
    unsafe { obj.Get(&prop, 0, &mut variant, None, None).ok()? };
    unsafe { Some(variant.Anonymous.Anonymous.Anonymous.lVal as f32) }
}

unsafe fn wmi_set_brightness_inner(percent: u32) -> bool {
    let services = match unsafe { wmi_connect() } {
        Some(s) => s,
        None => return false,
    };

    let lang = BSTR::from("WQL");
    let query = BSTR::from("SELECT * FROM WmiMonitorBrightnessMethods");
    let flags = WBEM_GENERIC_FLAG_TYPE(48);

    let class_enum = match unsafe { services.ExecQuery(&lang, &query, flags, None) } {
        Ok(e) => e,
        Err(_) => return false,
    };

    let mut obj: Option<IWbemClassObject> = None;
    let mut returned = 0u32;
    unsafe {
        if class_enum.Next(-1, std::slice::from_mut(&mut obj), &mut returned).is_err() {
            return false;
        }
    }

    let obj = match obj {
        Some(o) => o,
        None => return false,
    };

    let in_params = match unsafe { obj.SpawnInstance(0) } {
        Ok(p) => p,
        Err(_) => return false,
    };

    let timeout_prop = BSTR::from("Timeout");
    let timeout_val = VARIANT::from(0u32);
    unsafe { let _ = in_params.Put(&timeout_prop, 0, &timeout_val, 0); }

    let brightness_prop = BSTR::from("Brightness");
    let brightness_val = VARIANT::from(percent as u8);
    unsafe { let _ = in_params.Put(&brightness_prop, 0, &brightness_val, 0); }

    let method = BSTR::from("WmiSetBrightness");
    let obj_path = BSTR::from("WmiMonitorBrightnessMethods=@");
    unsafe {
        services.ExecMethod(&obj_path, &method, WBEM_GENERIC_FLAG_TYPE(0), None, Some(&in_params), None, None).is_ok()
    }
}

fn wmi_get_brightness() -> Option<f32> {
    unsafe { wmi_get_brightness_inner() }
}

fn wmi_set_brightness(percent: f32) -> bool {
    unsafe { wmi_set_brightness_inner(percent as u32) }
}

pub fn get_brightness() -> Option<f32> {
    if let Some(cached) = *BRIGHTNESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) {
        return Some(cached);
    }
    let value = wmi_get_brightness()?;
    *BRIGHTNESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(value);
    Some(value)
}

pub fn set_brightness(percent: f32) {
    let clamped = percent.clamp(0.0, 100.0);
    *BRIGHTNESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(clamped);
    std::thread::spawn(move || {
        wmi_set_brightness(clamped);
    });
}

#[allow(dead_code)]
pub fn sync_all() {
    if let Some(val) = wmi_get_brightness() {
        *BRIGHTNESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(val);
    }
}
