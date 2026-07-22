#![allow(dead_code, non_snake_case)]

use std::sync::{Mutex, mpsc};

use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

// ---------------------------------------------------------------------------
// Domain structs (WMI)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WmiMonitorBrightness {
    pub current_brightness: u8,
    pub instance_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiMonitorBrightnessEvent {
    #[allow(unused)]
    active: bool,
    brightness: u8,
    #[allow(unused)]
    instance_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WmiMonitorBrightnessMethods {
    #[serde(rename = "__Path")]
    __path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct WmiSetBrightnessPayload {
    timeout: u32,
    brightness: u8,
}

// ---------------------------------------------------------------------------
// Event channel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum BrightnessEvent {
    Changed(Vec<WmiMonitorBrightness>),
}

static BRIGHTNESS_EVENT_TX: Mutex<Option<mpsc::Sender<BrightnessEvent>>> = Mutex::new(None);

pub fn create_brightness_event_channel() -> mpsc::Receiver<BrightnessEvent> {
    let (tx, rx) = mpsc::channel();
    *BRIGHTNESS_EVENT_TX.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    rx
}

fn send_event(event: BrightnessEvent) {
    if let Some(tx) = BRIGHTNESS_EVENT_TX.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        let _ = tx.send(event);
    }
}

// ---------------------------------------------------------------------------
// WMI operations
// ---------------------------------------------------------------------------

fn wmi_query_brightness() -> Option<Vec<WmiMonitorBrightness>> {
    let wmi = WMIConnection::with_namespace_path("ROOT\\WMI").ok()?;
    wmi.query().ok()
}

fn wmi_set_brightness(percent: u32) -> bool {
    let wmi = match WMIConnection::with_namespace_path("ROOT\\WMI") {
        Ok(w) => w,
        Err(_) => return false,
    };

    let instances: Vec<WmiMonitorBrightnessMethods> = match wmi.query() {
        Ok(i) => i,
        Err(_) => return false,
    };

    let obj = match instances.first() {
        Some(o) => o,
        None => return false,
    };

    wmi.exec_instance_method::<WmiMonitorBrightnessMethods, ()>(
        obj.__path.clone(),
        "WmiSetBrightness",
        WmiSetBrightnessPayload {
            timeout: 0,
            brightness: percent as u8,
        },
    )
    .is_ok()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn get_brightness() -> Option<f32> {
    let results = wmi_query_brightness()?;
    results.first().map(|b| b.current_brightness as f32)
}

pub fn set_brightness(percent: f32) {
    let clamped = percent.clamp(0.0, 100.0);
    std::thread::spawn(move || {
        wmi_set_brightness(clamped as u32);
    });
}

pub fn sync_all() {
    if let Some(results) = wmi_query_brightness() {
        send_event(BrightnessEvent::Changed(results));
    }
}

/// Spawn a background thread that listens for WMI brightness change events.
pub fn start_event_listener() {
    std::thread::spawn(|| {
        let wmi = match WMIConnection::with_namespace_path("ROOT\\WMI") {
            Ok(w) => w,
            Err(_) => return,
        };

        // Send initial state
        if let Ok(results) = wmi.query::<WmiMonitorBrightness>() {
            send_event(BrightnessEvent::Changed(results));
        }

        // Listen for brightness change events
        for event_result in wmi
            .notification::<WmiMonitorBrightnessEvent>()
            .into_iter()
            .flatten()
        {
            let _ = event_result; // we don't care about the event payload, just that it fired
            if let Ok(results) = wmi.query::<WmiMonitorBrightness>() {
                send_event(BrightnessEvent::Changed(results));
            }
        }
    });
}
