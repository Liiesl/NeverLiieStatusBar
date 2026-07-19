use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
use windows::Win32::System::Power::{
    GetSystemPowerStatus, PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName,
    PowerSetActiveScheme, ACCESS_SCHEME, SYSTEM_POWER_STATUS,
};
use windows::Win32::System::WindowsProgramming::{
    AC_LINE_ONLINE, BATTERY_LIFE_UNKNOWN, BATTERY_PERCENTAGE_UNKNOWN,
};
use windows_core::GUID;

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

        let mut plans = Vec::new();
        let mut index = 0u32;

        loop {
            let mut guid_buf = [0u8; 16];
            let mut buf_size = 16u32;
            let result = PowerEnumerate(
                None,
                None,
                None,
                ACCESS_SCHEME,
                index,
                Some(guid_buf.as_mut_ptr()),
                &mut buf_size,
            );

            if result != ERROR_SUCCESS {
                break;
            }

            let scheme_guid: GUID = std::mem::transmute(guid_buf);
            let guid_str = guid_to_string(&scheme_guid);

            let name = {
                let mut name_buf = [0u16; 128];
                let mut name_size = (name_buf.len() * 2) as u32;
                let name_result = PowerReadFriendlyName(
                    None,
                    Some(&scheme_guid),
                    None,
                    None,
                    Some(name_buf.as_mut_ptr() as *mut u8),
                    &mut name_size,
                );

                if name_result == ERROR_SUCCESS {
                    let len = (name_size / 2) as usize;
                    String::from_utf16_lossy(&name_buf[..len])
                } else {
                    format!("Plan {}", index)
                }
            };

            let is_active = guid_str == active_guid_str;

            plans.push(PowerPlan {
                name,
                guid: guid_str,
                is_active,
            });

            index += 1;
        }

        plans
    }
}

pub fn set_power_plan(guid: &str) {
    unsafe {
        if let Ok(scheme_guid) = GUID::try_from(guid) {
            let _ = PowerSetActiveScheme(None, Some(&scheme_guid));
        }
    }
}
