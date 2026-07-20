use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ─── Settings Registry Definition ───────────────────────────────────────────

pub struct SettingDef {
    pub key: &'static str,
    pub desc: &'static str,
    pub default_str: &'static str,
    pub hidden: bool,
}

// ─── Declarative Configuration Macro ────────────────────────────────────────

macro_rules! define_settings {
    (
        $(
            $key:ident : $t:ty = $default:expr,
            desc: $desc:expr,
            hidden: $hidden:expr
            $( => $acc_name:ident : $acc_ty:ty = |$v:ident| $acc_expr:expr )? ;
        )*
    ) => {
        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct AppSettings {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $key: Option<$t>,
            )*
        }

        pub const SETTINGS: &[SettingDef] = &[
            $(
                SettingDef {
                    key: stringify!($key),
                    desc: $desc,
                    default_str: stringify!($default),
                    hidden: $hidden,
                },
            )*
        ];

        $(
            define_settings!(@accessor $key : $t = $default $( => $acc_name : $acc_ty = |$v| $acc_expr )? );
        )*

        $(
            define_settings!(@setter $key : $t );
        )*
    };

    (@accessor $key:ident : $t:ty = $default:expr => $acc_name:ident : $acc_ty:ty = |$v:ident| $acc_expr:expr) => {
        pub fn $acc_name() -> $acc_ty {
            let $v = settings_read().$key.clone().unwrap_or_else(|| $default);
            $acc_expr
        }
    };

    (@accessor $key:ident : $t:ty = $default:expr) => {
        pub fn $key() -> $t {
            settings_read().$key.clone().unwrap_or_else(|| $default)
        }
    };

    (@setter $key:ident : $t:ty) => {
        paste::paste! {
            #[allow(dead_code)]
            pub fn [<set_ $key>](val: $t) {
                update_setting(|s| s.$key = Some(val));
            }
        }
    };
}

// ─── Settings Declaration ───────────────────────────────────────────────────

define_settings! {
    bar_height: f32 = 40.0,
        desc: "Height of the status bar in pixels.",
        hidden: true;

    mouse_trigger_height: f32 = 5.0,
        desc: "Height of the edge hover zone in pixels.",
        hidden: false;

    floating_margin_x: f32 = 20.0,
        desc: "Horizontal margin of the status bar.",
        hidden: true;

    anim_duration_ms: u64 = 300,
        desc: "Duration of the slide animation in milliseconds.",
        hidden: false
        => anim_duration: Duration = |v| Duration::from_millis(v);

    monitor_interval_ms: u64 = 200,
        desc: "Polling rate for cursor position, tray events, z-order (ms).",
        hidden: true
        => monitor_interval: Duration = |v| Duration::from_millis(v);

    auto_hide_delay_ms: u64 = 400,
        desc: "Delay before the bar hides after losing focus (ms).",
        hidden: false
        => auto_hide_delay: Duration = |v| Duration::from_millis(v);

    trigger_dwell_ms: u64 = 500,
        desc: "How long the cursor must dwell at the top edge before the bar reappears (ms).",
        hidden: false
        => trigger_dwell_time: Duration = |v| Duration::from_millis(v);

    popup_width: f32 = 320.0,
        desc: "Width of popup panels in pixels.",
        hidden: true;

    popup_min_height: f32 = 400.0,
        desc: "Minimum height of popup panels in pixels.",
        hidden: true;

    audio_poll_rate_ms: u64 = 1000,
        desc: "Media state polling rate in milliseconds.",
        hidden: true
        => audio_poll_rate: Duration = |v| Duration::from_millis(v);

    bg_color: [f32; 4] = [25.0 / 255.0, 25.0 / 255.0, 25.0 / 255.0, 1.0],
        desc: "Background color as [r, g, b, a] (0.0-1.0).",
        hidden: true;

    border_color: [f32; 4] = [50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0],
        desc: "Border color as [r, g, b, a] (0.0-1.0).",
        hidden: true;

    text_color: [f32; 4] = [0.878, 0.878, 0.878, 1.0],
        desc: "Text color as [r, g, b, a] (0.0-1.0).",
        hidden: true;

    hover_bg: [f32; 4] = [1.0, 1.0, 1.0, 30.0 / 255.0],
        desc: "Hover background color as [r, g, b, a] (0.0-1.0).",
        hidden: true;

    settings_file: String = "NeverLiieStatusBar-settings.toml".to_string(),
        desc: "Name of the settings file.",
        hidden: true;
}

// ─── Loading and Saving Logic ───────────────────────────────────────────────

impl AppSettings {
    fn load_from_disk() -> Self {
        let path = settings_path();
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => {
                let _ = fs::write(&path, Self::generate_template());
                return Self::default();
            }
        };
        match toml::from_str::<Self>(&data) {
            Ok(s) => s,
            Err(_) => Self::default(),
        }
    }

    fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let mut output = Self::generate_template();
            if !content.trim().is_empty() {
                output.push_str("# -----------------------------------\n");
                output.push_str("# Active overrides:\n\n");
                output.push_str(&content);
            }
            let _ = fs::write(path, output);
        }
    }

    fn generate_template() -> String {
        let mut out = String::from(
            "# NeverLiieStatusBar Settings\n\
             #\n\
             # Uncomment and change only the values you want to override.\n\
             # If a line is deleted or left blank, the default is used.\n\
             # Unknown keys from newer versions are silently ignored.\n",
        );
        for s in SETTINGS {
            if s.hidden {
                continue;
            }
            out.push_str(&format!(
                "\n# {}\n# Default: {}\n#{} =\n",
                s.desc, s.default_str, s.key,
            ));
        }
        out
    }
}

// ─── Global State and Path Helpers ──────────────────────────────────────────

static APP_SETTINGS: OnceLock<RwLock<AppSettings>> = OnceLock::new();

pub fn init_settings() {
    let settings = AppSettings::load_from_disk();
    let _ = APP_SETTINGS.set(RwLock::new(settings));
}

fn settings_read() -> std::sync::RwLockReadGuard<'static, AppSettings> {
    APP_SETTINGS
        .get()
        .expect("settings not initialized")
        .read()
        .unwrap()
}

pub fn update_setting(f: impl FnOnce(&mut AppSettings)) {
    let lock = APP_SETTINGS.get().expect("settings not initialized");
    let mut s = lock.write().unwrap();
    f(&mut s);
    s.save();
}

pub fn open_config_folder() {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::process::Command::new("explorer")
            .arg(parent)
            .spawn();
    }
}

fn settings_path() -> PathBuf {
    let app_data = std::env::var("APPDATA")
        .unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(app_data).join("NeverLiieStatusBar");
    let _ = fs::create_dir_all(&dir);
    dir.join("NeverLiieStatusBar-settings.toml")
}
