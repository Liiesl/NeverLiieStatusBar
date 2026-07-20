use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
use windows::Win32::System::Power::{
    GetSystemPowerStatus, PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName,
    PowerSetActiveScheme, ACCESS_SCHEME, SYSTEM_POWER_STATUS,
};
use windows::Win32::System::WindowsProgramming::{
    AC_LINE_ONLINE, BATTERY_LIFE_UNKNOWN, BATTERY_PERCENTAGE_UNKNOWN,
};
use windows_core::GUID;

windows_core::link!(
    "powrprof.dll" "system" fn PowerGetEffectiveOverlayScheme(
        powermodeguid: *mut GUID
    ) -> windows::Win32::Foundation::WIN32_ERROR
);

windows_core::link!(
    "powrprof.dll" "system" fn PowerSetActiveOverlayScheme(
        powermodeguid: *const GUID
    ) -> windows::Win32::Foundation::WIN32_ERROR
);

const GUID_POWER_MODE_NONE: GUID = GUID {
    data1: 0x00000000,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};
const GUID_POWER_MODE_BEST_EFFICIENCY: GUID = GUID {
    data1: 0x961cc777,
    data2: 0x2547,
    data3: 0x4f9d,
    data4: [0x81, 0x74, 0x7d, 0x86, 0x18, 0x1b, 0x8a, 0x7a],
};
const GUID_POWER_MODE_BEST_PERFORMANCE: GUID = GUID {
    data1: 0xded574b5,
    data2: 0x45a0,
    data3: 0x4f42,
    data4: [0x87, 0x37, 0x46, 0x34, 0x5c, 0x09, 0xc2, 0x38],
};

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percent: u8,
    pub is_plugged: bool,
    pub secs_left: i32, // -1 = unknown, -2 = unlimited
}

#[derive(Debug, Clone)]
pub struct PowerPlan {
    pub name: String,
    pub guid: String,
    pub is_active: bool,
}

fn guid_to_string(guid: &GUID) -> String {
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7]
    )
}

unsafe fn read_scheme_name(guid: GUID) -> String {
    let mut name_buf = [0u16; 128];
    let mut name_size = (name_buf.len() * 2) as u32;
    let result = unsafe {
        PowerReadFriendlyName(
            None,
            Some(&guid),
            None,
            None,
            Some(name_buf.as_mut_ptr() as *mut u8),
            &mut name_size,
        )
    };
    if result == ERROR_SUCCESS {
        let len = (name_size / 2) as usize;
        String::from_utf16_lossy(&name_buf[..len])
    } else {
        String::new()
    }
}

fn is_overlay_guid(guid: &GUID) -> bool {
    matches!(
        *guid,
        GUID_POWER_MODE_NONE | GUID_POWER_MODE_BEST_EFFICIENCY | GUID_POWER_MODE_BEST_PERFORMANCE
    )
}

fn overlay_guid_name(guid: &GUID) -> &'static str {
    if *guid == GUID_POWER_MODE_BEST_EFFICIENCY {
        "Best Power Efficiency"
    } else if *guid == GUID_POWER_MODE_BEST_PERFORMANCE {
        "Best Performance"
    } else {
        "Balanced"
    }
}

unsafe fn get_active_overlay() -> Option<GUID> {
    let mut guid = GUID::default();
    if unsafe { PowerGetEffectiveOverlayScheme(&mut guid) } == ERROR_SUCCESS {
        Some(guid)
    } else {
        None
    }
}

unsafe fn enumerate_scheme_guids() -> Vec<GUID> {
    let mut guids = Vec::new();
    let mut index = 0u32;
    loop {
        let mut guid_buf = [0u8; 16];
        let mut buf_size = 16u32;
        if unsafe { PowerEnumerate(None, None, None, ACCESS_SCHEME, index, Some(guid_buf.as_mut_ptr()), &mut buf_size) } != ERROR_SUCCESS {
            break;
        }
        guids.push(unsafe { std::mem::transmute(guid_buf) });
        index += 1;
    }
    guids
}

pub fn get_battery_info() -> Option<BatteryInfo> {
    unsafe {
        let mut status = SYSTEM_POWER_STATUS::default();
        GetSystemPowerStatus(&mut status).ok()?;

        let percent = if status.BatteryLifePercent == BATTERY_PERCENTAGE_UNKNOWN as u8 {
            0
        } else {
            status.BatteryLifePercent
        };

        let is_plugged = status.ACLineStatus == AC_LINE_ONLINE as u8;

        let secs_left = if status.BatteryLifeTime == BATTERY_LIFE_UNKNOWN {
            -1
        } else if status.BatteryLifeTime == 0 {
            -2
        } else {
            status.BatteryLifeTime as i32
        };

        Some(BatteryInfo {
            percent,
            is_plugged,
            secs_left,
        })
    }
}

pub fn get_power_plans() -> Vec<PowerPlan> {
    unsafe {
        let overlay_guids = [
            GUID_POWER_MODE_BEST_PERFORMANCE,
            GUID_POWER_MODE_NONE,
            GUID_POWER_MODE_BEST_EFFICIENCY,
        ];

        if let Some(active_overlay) = get_active_overlay() {
            return overlay_guids
                .iter()
                .map(|&guid| {
                    let name = overlay_guid_name(&guid).to_string();
                    let is_active = active_overlay == guid;
                    PowerPlan {
                        name,
                        guid: guid_to_string(&guid),
                        is_active,
                    }
                })
                .collect();
        }

        let active_guid_str = {
            let mut guid_ptr: *mut GUID = std::ptr::null_mut();
            if PowerGetActiveScheme(None, &mut guid_ptr) == ERROR_SUCCESS && !guid_ptr.is_null() {
                let s = guid_to_string(&*guid_ptr);
                let _ = LocalFree(Some(HLOCAL(guid_ptr as *mut _)));
                s
            } else {
                String::new()
            }
        };

        enumerate_scheme_guids()
            .into_iter()
            .enumerate()
            .map(|(i, guid)| {
                let name = read_scheme_name(guid);
                let guid_str = guid_to_string(&guid);
                PowerPlan {
                    name: if name.is_empty() { format!("Plan {}", i) } else { name },
                    guid: guid_str.clone(),
                    is_active: guid_str == active_guid_str,
                }
            })
            .collect()
    }
}

pub fn set_power_plan(guid: &str) {
    unsafe {
        if let Ok(scheme_guid) = GUID::try_from(guid) {
            if is_overlay_guid(&scheme_guid) {
                let _ = PowerSetActiveOverlayScheme(&scheme_guid);
            } else {
                let _ = PowerSetActiveScheme(None, Some(&scheme_guid));
            }
        }
    }
}
