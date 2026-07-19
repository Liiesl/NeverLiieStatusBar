use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::WindowsProgramming::{
    AC_LINE_ONLINE, BATTERY_LIFE_UNKNOWN, BATTERY_PERCENTAGE_UNKNOWN,
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
    let output = match Command::new("powercfg")
        .args(["/list"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut plans = Vec::new();

    for line in stdout.lines() {
        if !line.contains("GUID") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let guid = parts[2].to_string();

        // Extract name between parentheses
        if let Some(start) = line.find('(')
            && let Some(end) = line.find(')') {
                let name = line[start + 1..end].to_string();
                let is_active = line.contains('*');
                plans.push(PowerPlan {
                    name,
                    guid,
                    is_active,
                });
            }
    }

    plans
}

pub fn set_power_plan(guid: &str) {
    let _ = Command::new("powercfg")
        .args(["/setactive", guid])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
}
