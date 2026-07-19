use std::time::Duration;

pub const BAR_HEIGHT: f32 = 40.0;
pub const MOUSE_TRIGGER_HEIGHT: f32 = 5.0;

pub const FLOATING_MARGIN_X: f32 = 20.0;
pub const FLOATING_MARGIN_Y: f32 = 0.0;

pub const ANIM_DURATION: Duration = Duration::from_millis(300);
pub const MONITOR_INTERVAL: Duration = Duration::from_millis(200);
pub const AUTO_HIDE_DELAY: Duration = Duration::from_millis(400);
pub const TRIGGER_DWELL_TIME: Duration = Duration::from_millis(500);


pub const BG_COLOR: [f32; 4] = [25.0 / 255.0, 25.0 / 255.0, 25.0 / 255.0, 1.0];
pub const BORDER_COLOR: [f32; 4] = [50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0];
pub const TEXT_COLOR: [f32; 4] = [0.878, 0.878, 0.878, 1.0];
pub const HOVER_BG: [f32; 4] = [255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 30.0 / 255.0];

pub const POPUP_WIDTH: f32 = 320.0;
pub const POPUP_MIN_HEIGHT: f32 = 400.0;

pub const AUDIO_POLL_RATE: Duration = Duration::from_secs(1);
