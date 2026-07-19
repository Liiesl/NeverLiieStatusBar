use std::sync::{Mutex, OnceLock};
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::Win32::NetworkManagement::WiFi::*;
use windows::Win32::Foundation::HANDLE;
use windows::Devices::Radios::{Radio, RadioKind, RadioState};

#[derive(Debug, Clone)]
pub struct WirelessState {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_enabled: bool,
    pub battery_saver_enabled: bool,
}

static WIRELESS_STATE: Mutex<Option<WirelessState>> = Mutex::new(None);

fn async_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

fn block_on_async<F: std::future::Future>(f: F) -> F::Output {
    async_runtime().block_on(f)
}

fn is_wifi_connected() -> bool {
    unsafe {
        let mut handle = HANDLE::default();
        let mut version = 0u32;
        if WlanOpenHandle(2, None, &mut version, &mut handle) != 0 {
            return false;
        }

        let mut interface_list = std::ptr::null_mut();
        let result = WlanEnumInterfaces(handle, None, &mut interface_list);
        if result != 0 {
            let _ = WlanCloseHandle(handle, None);
            return false;
        }

        let list = &*interface_list;
        let count = list.dwNumberOfItems as usize;
        let connected = (0..count).any(|i| {
            let info = &*list.InterfaceInfo.as_ptr().add(i);
            info.isState.0 == 1 // wlan_interface_state_connected
        });

        WlanFreeMemory(interface_list as *const _);
        let _ = WlanCloseHandle(handle, None);
        connected
    }
}

async fn get_radio_state(kind: RadioKind) -> Option<RadioState> {
    let radios = Radio::GetRadiosAsync().ok()?.await.ok()?;
    for i in 0..radios.Size().ok()? {
        let radio = radios.GetAt(i).ok()?;
        if radio.Kind().ok()? == kind {
            return radio.State().ok();
        }
    }
    None
}

async fn set_radio_state(kind: RadioKind, new_state: RadioState) -> bool {
    let radios = match Radio::GetRadiosAsync() {
        Ok(op) => match op.await {
            Ok(r) => r,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    for i in 0..radios.Size().unwrap_or(0) {
        if let Ok(radio) = radios.GetAt(i) {
            if radio.Kind().unwrap_or(RadioKind(0)) == kind {
                if let Ok(op) = radio.SetStateAsync(new_state) {
                    return op.await.map(|_| true).unwrap_or(false);
                }
            }
        }
    }
    false
}

async fn set_all_radios(new_state: RadioState) -> bool {
    let radios = match Radio::GetRadiosAsync() {
        Ok(op) => match op.await {
            Ok(r) => r,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    let mut any_changed = false;
    for i in 0..radios.Size().unwrap_or(0) {
        if let Ok(radio) = radios.GetAt(i) {
            let kind = radio.Kind().unwrap_or(RadioKind(0));
            if kind == RadioKind::WiFi || kind == RadioKind::Bluetooth || kind == RadioKind::MobileBroadband {
                if let Ok(op) = radio.SetStateAsync(new_state) {
                    if op.await.is_ok() {
                        any_changed = true;
                    }
                }
            }
        }
    }
    any_changed
}

fn is_battery_saver_enabled() -> bool {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance -Namespace root\\cimv2 -ClassName Win32_Battery).BatteryStatus",
        ])
        .creation_flags(0x08000000)
        .output()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            s == "2"
        })
        .unwrap_or(false)
}

fn set_battery_saver_enable(enable: bool) {
    let guid = if enable {
        "a1841308-3541-4fab-bc81-f71556f20b4a"
    } else {
        "381b4222-f694-41f0-9685-ff5bb260df2e"
    };
    let _ = Command::new("powercfg")
        .args(["-setactive", guid])
        .creation_flags(0x08000000)
        .output();
}

pub fn sync_all() {
    let wifi_connected = is_wifi_connected();
    let wifi_radio = block_on_async(get_radio_state(RadioKind::WiFi))
        .map(|s| s == RadioState::On)
        .unwrap_or(false);
    let bluetooth_on = block_on_async(get_radio_state(RadioKind::Bluetooth))
        .map(|s| s == RadioState::On)
        .unwrap_or(false);

    let airplane = {
        let wifi_off = block_on_async(get_radio_state(RadioKind::WiFi))
            .map(|s| s != RadioState::On)
            .unwrap_or(true);
        let bt_off = block_on_async(get_radio_state(RadioKind::Bluetooth))
            .map(|s| s != RadioState::On)
            .unwrap_or(true);
        wifi_off && bt_off
    };

    let state = WirelessState {
        wifi_enabled: wifi_connected || wifi_radio,
        bluetooth_enabled: bluetooth_on,
        airplane_enabled: airplane,
        battery_saver_enabled: is_battery_saver_enabled(),
    };
    *WIRELESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(state);
}

pub fn get_state() -> WirelessState {
    {
        let guard = WIRELESS_STATE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = guard.as_ref() {
            return s.clone();
        }
    }
    sync_all();
    WIRELESS_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or(WirelessState {
            wifi_enabled: false,
            bluetooth_enabled: false,
            airplane_enabled: false,
            battery_saver_enabled: false,
        })
}

pub fn toggle_wifi() -> bool {
    let current = get_state();
    let new_radio_state = if current.wifi_enabled {
        RadioState::Off
    } else {
        RadioState::On
    };
    let result = block_on_async(set_radio_state(RadioKind::WiFi, new_radio_state));
    if result {
        let new_connected = !current.wifi_enabled && is_wifi_connected();
        let new_enabled = new_radio_state == RadioState::On || new_connected;
        if let Some(s) = &mut *WIRELESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) {
            s.wifi_enabled = new_enabled;
        }
        return new_enabled;
    }
    current.wifi_enabled
}

pub fn toggle_bluetooth() -> bool {
    let current = get_state();
    let new_state = if current.bluetooth_enabled {
        RadioState::Off
    } else {
        RadioState::On
    };
    let result = block_on_async(set_radio_state(RadioKind::Bluetooth, new_state));
    if result {
        if let Some(s) = &mut *WIRELESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) {
            s.bluetooth_enabled = new_state == RadioState::On;
        }
        return new_state == RadioState::On;
    }
    current.bluetooth_enabled
}

pub fn toggle_airplane() -> bool {
    let current = get_state();
    let new_state = if current.airplane_enabled {
        RadioState::On
    } else {
        RadioState::Off
    };
    let result = block_on_async(set_all_radios(new_state));
    if result {
        if let Some(s) = &mut *WIRELESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) {
            s.airplane_enabled = new_state == RadioState::Off;
            if new_state == RadioState::Off {
                s.wifi_enabled = false;
                s.bluetooth_enabled = false;
            } else {
                s.wifi_enabled = is_wifi_connected();
                s.bluetooth_enabled = block_on_async(get_radio_state(RadioKind::Bluetooth))
                    .map(|s| s == RadioState::On)
                    .unwrap_or(false);
            }
        }
        return new_state == RadioState::Off;
    }
    current.airplane_enabled
}

pub fn toggle_battery_saver() -> bool {
    let current = get_state();
    let new_val = !current.battery_saver_enabled;
    set_battery_saver_enable(new_val);
    if let Some(s) = &mut *WIRELESS_STATE.lock().unwrap_or_else(|e| e.into_inner()) {
        s.battery_saver_enabled = new_val;
    }
    new_val
}
