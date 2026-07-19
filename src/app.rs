use std::collections::BTreeMap;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use iced::window::{self, Id as WindowId};
use iced::{Element, Point, Size, Subscription, Task, time};

use crate::audio_control;
use crate::bar_ui;
use crate::battery_control;
use crate::brightness_control;
use crate::config;
use crate::ipc::{TrayIpcServer, Win32TrayEvent};
use crate::keyboard_control;
use crate::network;
use crate::popup::PopupKind;
use crate::profile_control;
use crate::systray::{SysTrayIconId, SystemTrayManager, TrayIconAction};
use crate::wireless_control;
use crate::win32;

#[derive(Debug, Clone)]
pub enum PowerAction {
    Lock,
    Sleep,
    Restart,
    Shutdown,
    Quit,
    OpenSettings,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    MonitorTick,
    Frame(Instant),
    WindowOpened(WindowId),
    HwndReady(WindowId, u64),
    MonitorSizeReady(WindowId, Size),
    WindowClosed(WindowId),
    ClockTick(String),
    OpenPopup { kind: PopupKind },
    ClosePopup,
    PopupWindowOpened(WindowId),
    PopupHwndReady(WindowId, u64),
    BarMouseEnter,
    BarMouseExit,
    PopupMouseEnter,
    PopupMouseExit,
    TrayIconClicked { id: SysTrayIconId, action: TrayIconAction },

    SettingsSpeakerVolume(f32),
    SettingsMicVolume(f32),
    SettingsBrightness(f32),
    ToggleSpeakerMute,
    ToggleMicMute,
    AudioScanDevices,
    AudioScanResult(Vec<audio_control::AudioDevice>, Vec<audio_control::AudioDevice>),
    AudioSelectDevice { device_id: String, is_input: bool },
    AudioSelectDeviceResult(bool),
    MediaTick,
    MediaTogglePlay,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStateResult(audio_control::MediaState),
    ToggleWifi,
    ToggleBluetooth,
    ToggleAirplane,
    ToggleBatterySaver,
    PowerAction(PowerAction),
    SyncSettings,
    NetworkStatusTick,
    NetworkStatusResult(bool),
    NetworkScan,
    NetworkScanResult(Vec<network::NetworkInfo>),
    NetworkPasswordChanged(String),
    NetworkToggleExpand(String),
    NetworkConnect { ssid: String, password: String },
    NetworkConnectResult { ssid: String, ok: bool },
    NetworkDisconnect,
    NetworkDisconnectResult(bool),
    BatteryTick,
    BatteryPollResult(Option<battery_control::BatteryInfo>),
    PowerPlanList,
    PowerPlanListResult(Vec<battery_control::PowerPlan>),
    PowerPlanSwitch(String),
    KeyboardTick,
    KeyboardPollResult(Option<keyboard_control::KeyboardLayout>),
    KeyboardLayoutList,
    KeyboardLayoutListResult(Vec<keyboard_control::KeyboardLayout>),
    KeyboardSwitchLayout(usize),
    ProfileLoadResult(profile_control::ProfileInfo),
    ProfileOpenLauncher,
    UpdateCheckResult(Box<Option<velopack::UpdateInfo>>),
    UpdateCheckAgain,
    UpdateDownloadStart,
    UpdateApply,
    UpdateDismiss,
}

#[derive(Debug)]
pub enum WindowKind {
    Bar,
    Popup { kind: PopupKind },
}

#[derive(Debug)]
pub struct State {
    pub visible: bool,
    pub current_y: f32,
    pub slide: Option<SlideAnim>,
    pub edge_dwell_start: Option<Instant>,
    pub hide_timer_start: Option<Instant>,
    pub cursor_pos: Point,
    pub bar_hovered: bool,
    pub popup_hovered: bool,
    pub windows: BTreeMap<WindowId, WindowKind>,
    pub bar_hwnd: u64,
    pub screen_width: f32,
    pub initialized: bool,
    pub clock_text: String,
    pub tray_manager: SystemTrayManager,
    pub tray_rx: Option<mpsc::Receiver<Win32TrayEvent>>,
    pub _tray_server: Option<TrayIpcServer>,

    pub speaker_volume: f32,
    pub mic_volume: f32,
    pub brightness: f32,
    pub speaker_muted: bool,
    pub mic_muted: bool,
    pub output_devices: Vec<audio_control::AudioDevice>,
    pub input_devices: Vec<audio_control::AudioDevice>,
    pub current_output_device_id: Option<String>,
    pub current_input_device_id: Option<String>,
    pub media_title: String,
    pub media_artist: String,
    pub media_thumbnail: Vec<u8>,
    pub media_is_playing: bool,
    pub media_has_session: bool,
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_enabled: bool,
    pub battery_saver_enabled: bool,
    #[allow(dead_code)]
    pub settings_synced: bool,
    pub is_online: bool,
    pub networks: Vec<network::NetworkInfo>,
    pub network_status: String,
    pub wifi_password_input: String,
    pub expanded_network: Option<String>,
    pub battery_percent: u8,
    pub battery_is_plugged: bool,
    pub battery_secs_left: i32,
    pub battery_plans: Vec<battery_control::PowerPlan>,
    pub keyboard_lang_text: String,
    pub keyboard_layouts: Vec<keyboard_control::KeyboardLayout>,
    pub current_keyboard_hkl: Option<usize>,
    pub last_real_hwnd: u64,

    pub profile_display_name: String,
    pub profile_principal_name: String,
    pub profile_avatar_handle: Option<iced::widget::image::Handle>,
    pub profile_loaded: bool,

    pub update_info: Option<velopack::UpdateInfo>,
    pub update_downloading: bool,
    pub update_download_progress: i16,
    pub update_ready: bool,
    pub update_rx: Option<Arc<std::sync::Mutex<std::sync::mpsc::Receiver<i16>>>>,
}

#[derive(Debug, Clone)]
pub struct SlideAnim {
    pub start_time: Instant,
    pub from_y: f32,
    pub to_y: f32,
}

impl State {
    fn is_at_top_edge(&self) -> bool {
        self.cursor_pos.y >= 0.0 && self.cursor_pos.y < config::MOUSE_TRIGGER_HEIGHT
    }

    fn ease_out_cubic(t: f32) -> f32 {
        let inv = 1.0 - t;
        1.0 - inv * inv * inv
    }
}

pub fn boot() -> (State, Task<Message>) {
    let (tray_server, tray_rx) = TrayIpcServer::start();

    wireless_control::sync_all();
    let wireless = wireless_control::get_state();

    let battery = battery_control::get_battery_info();
    let kb_layout = keyboard_control::get_active_layout();

    let state = State {
        visible: true,
        current_y: 0.0,
        slide: None,
        edge_dwell_start: None,
        hide_timer_start: None,
        cursor_pos: Point::new(-1.0, -1.0),
        bar_hovered: false,
        popup_hovered: false,
        windows: BTreeMap::new(),
        bar_hwnd: 0,
        screen_width: 800.0,
        initialized: false,
        clock_text: String::new(),
        tray_manager: SystemTrayManager::new(),
        tray_rx: Some(tray_rx),
        _tray_server: Some(tray_server),

        speaker_volume: audio_control::get_speaker_volume(),
        mic_volume: audio_control::get_mic_volume(),
        brightness: brightness_control::get_brightness().unwrap_or(75.0),
        speaker_muted: audio_control::is_speaker_muted(),
        mic_muted: audio_control::is_mic_muted(),
        output_devices: Vec::new(),
        input_devices: Vec::new(),
        current_output_device_id: audio_control::get_current_output_device_id(),
        current_input_device_id: audio_control::get_current_input_device_id(),
        media_title: String::new(),
        media_artist: String::new(),
        media_thumbnail: Vec::new(),
        media_is_playing: false,
        media_has_session: false,
        wifi_enabled: wireless.wifi_enabled,
        bluetooth_enabled: wireless.bluetooth_enabled,
        airplane_enabled: wireless.airplane_enabled,
        battery_saver_enabled: wireless.battery_saver_enabled,
        settings_synced: true,
        is_online: network::check_internet(),
        networks: Vec::new(),
        network_status: "Initializing...".to_string(),
        wifi_password_input: String::new(),
        expanded_network: None,
        battery_percent: battery.as_ref().map(|b| b.percent).unwrap_or(0),
        battery_is_plugged: battery.as_ref().map(|b| b.is_plugged).unwrap_or(false),
        battery_secs_left: battery.as_ref().map(|b| b.secs_left).unwrap_or(-1),
        battery_plans: Vec::new(),
        keyboard_lang_text: kb_layout
            .as_ref()
            .map(|l| keyboard_control::get_bar_text(l.hkl_raw))
            .unwrap_or_else(|| "EN".to_string()),
        keyboard_layouts: Vec::new(),
        current_keyboard_hkl: kb_layout.map(|l| l.hkl_raw),
        last_real_hwnd: 0,

        profile_display_name: "Loading...".to_string(),
        profile_principal_name: String::new(),
        profile_avatar_handle: None,
        profile_loaded: false,

        update_info: None,
        update_downloading: false,
        update_download_progress: 0,
        update_ready: false,
        update_rx: None,
    };

    let window_settings = window::Settings {
        size: Size::new(800.0, config::BAR_HEIGHT),
        position: window::Position::Specific(Point::new(0.0, 0.0)),
        decorations: false,
        transparent: true,
        level: window::Level::AlwaysOnTop,
        resizable: false,
        visible: true,
        exit_on_close_request: false,
        ..Default::default()
    };

    let (_id, open_task) = window::open(window_settings);

    let profile_task = Task::perform(
        async {
            tokio::task::spawn_blocking(profile_control::get_profile_info)
                .await
                .unwrap_or_else(|_| profile_control::ProfileInfo {
                    display_name: "User".to_string(),
                    principal_name: String::new(),
                    avatar: None,
                })
        },
        Message::ProfileLoadResult,
    );

    let update_check_task = Task::perform(
        async {
            tokio::task::spawn_blocking(crate::updater::check_for_updates)
                .await
                .unwrap_or(None)
        },
        |info| Message::UpdateCheckResult(Box::new(info)),
    );

    (state, Task::batch([open_task.map(Message::WindowOpened), profile_task, update_check_task]))
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::WindowOpened(id) => {
            let is_bar = state.windows.is_empty();
            if is_bar {
                state.windows.insert(id, WindowKind::Bar);
            }
            Task::batch([
                window::raw_id::<Message>(id).map(move |raw| Message::HwndReady(id, raw)),
                window::monitor_size(id).map(move |opt_size| {
                    let size = opt_size.unwrap_or(Size::new(1920.0, 1080.0));
                    Message::MonitorSizeReady(id, size)
                }),
            ])
        }
        Message::HwndReady(id, raw_hwnd) => {
            if let Some(WindowKind::Bar) = state.windows.get(&id) {
                state.bar_hwnd = raw_hwnd;
                win32::apply_window_flags(raw_hwnd);
                win32::apply_dwm_rounded_corners(raw_hwnd);
            } else {
                win32::apply_popup_flags(raw_hwnd);
                win32::apply_dwm_rounded_corners(raw_hwnd);
            }
            Task::none()
        }
        Message::MonitorSizeReady(id, monitor_size) => {
            state.screen_width = monitor_size.width;
            state.initialized = true;

            Task::batch([
                window::resize(id, Size::new(monitor_size.width, config::BAR_HEIGHT)),
                window::move_to(id, Point::new(0.0, 0.0)),
            ])
        }
        Message::WindowClosed(id) => {
            state.windows.remove(&id);
            if state.windows.is_empty() {
                iced::exit()
            } else {
                Task::none()
            }
        }
        Message::BarMouseEnter => {
            state.bar_hovered = true;
            Task::none()
        }
        Message::BarMouseExit => {
            state.bar_hovered = false;
            Task::none()
        }
        Message::PopupMouseEnter => {
            state.popup_hovered = true;
            Task::none()
        }
        Message::PopupMouseExit => {
            state.popup_hovered = false;
            Task::none()
        }
        Message::MonitorTick => {
            if !state.initialized {
                return Task::none();
            }

            // Poll tray events from the IPC receiver
            if let Some(rx) = &state.tray_rx {
                while let Ok(event) = rx.try_recv() {
                    state.tray_manager.handle_event(event);
                }
            }

            // Poll global cursor position via Win32 API for edge dwell detection
            if let Some((sx, sy)) = win32::get_cursor_pos() {
                state.cursor_pos = Point::new(sx, sy);
            }

            // Track the real foreground window (not ours) for keyboard layout switching
            let real_hwnd = keyboard_control::get_real_foreground_hwnd();
            if real_hwnd != 0 {
                state.last_real_hwnd = real_hwnd;
            }

            if state.visible {
                win32::force_z_order(state.bar_hwnd);
                win32::apply_window_flags(state.bar_hwnd);
            }

            let is_at_top = state.is_at_top_edge();
            let is_hovering = state.bar_hovered || state.popup_hovered;

            // Edge dwell: if bar is hidden and cursor is at top edge,
            // require dwell time before triggering
            let trigger_activated = if is_at_top {
                if state.visible {
                    state.edge_dwell_start = None;
                    true
                } else {
                    if state.edge_dwell_start.is_none() {
                        state.edge_dwell_start = Some(Instant::now());
                    }
                    match state.edge_dwell_start {
                        Some(start) => start.elapsed() >= config::TRIGGER_DWELL_TIME,
                        None => false,
                    }
                }
            } else {
                state.edge_dwell_start = None;
                false
            };

            let user_interacting = is_hovering || trigger_activated;

            // Foreground window blocking
            let blocked = win32::is_foreground_blocked(state.bar_hwnd);

            let should_show = user_interacting || !blocked;

            if should_show {
                state.hide_timer_start = None;
                if !state.visible && state.slide.is_none() {
                    let from_y = state.current_y;
                    state.slide = Some(SlideAnim {
                        start_time: Instant::now(),
                        from_y,
                        to_y: 0.0,
                    });
                    state.visible = true;
                }
            } else {
                if state.visible && state.hide_timer_start.is_none() {
                    state.hide_timer_start = Some(Instant::now());
                }
                if let Some(start) = state.hide_timer_start
                    && start.elapsed() >= config::AUTO_HIDE_DELAY {
                        state.hide_timer_start = None;
                        if state.visible && state.slide.is_none() {
                            state.visible = false;
                            let from_y = state.current_y;
                            state.slide = Some(SlideAnim {
                                start_time: Instant::now(),
                                from_y,
                                to_y: -config::BAR_HEIGHT,
                            });
                        }
                        return close_all_popups(state);
                    }
            }
            if state.update_downloading
                && let Some(rx_arc) = state.update_rx.clone() {
                    let result = rx_arc.lock().unwrap().try_recv();
                    match result {
                        Ok(val) => {
                            state.update_download_progress = val;
                            if val >= 100 {
                                state.update_downloading = false;
                                state.update_ready = true;
                                state.update_rx = None;
                            }
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            state.update_downloading = false;
                            state.update_ready = true;
                            state.update_rx = None;
                        }
                        _ => {}
                    }
                }
            Task::none()
        }
        Message::Frame(now) => {
            if let Some(slide) = &state.slide {
                let elapsed = now.duration_since(slide.start_time);
                let total = config::ANIM_DURATION;
                let t = if total.as_millis() == 0 {
                    1.0
                } else {
                    (elapsed.as_secs_f32() / total.as_secs_f32()).min(1.0)
                };
                let eased = State::ease_out_cubic(t);
                state.current_y = slide.from_y + (slide.to_y - slide.from_y) * eased;

                if t >= 1.0 {
                    state.current_y = slide.to_y;
                    state.slide = None;
                }

                if let Some((id, _)) = state.windows.iter().find(|(_, kind)| matches!(kind, WindowKind::Bar)) {
                    return window::move_to(*id, Point::new(0.0, state.current_y));
                }
            }
            Task::none()
        }
        Message::ClockTick(text) => {
            state.clock_text = text;
            Task::none()
        }
        Message::OpenPopup { kind } => {
            let already_open = state.windows.values().any(|k| matches!(k, WindowKind::Popup { kind: k } if *k == kind));
            if already_open {
                return close_all_popups(state);
            }

            let close_task = close_all_popups(state);

            let is_left = matches!(kind, PopupKind::Profile | PopupKind::Update);
            let target_x = if is_left {
                let comp_center = config::FLOATING_MARGIN_X + 20.0 + 40.0;
                (comp_center - config::POPUP_WIDTH / 2.0).max(10.0)
            } else {
                let comp_center = state.screen_width - config::FLOATING_MARGIN_X - 20.0 - 80.0;
                let x = comp_center - config::POPUP_WIDTH / 2.0;
                if x + config::POPUP_WIDTH > state.screen_width - 10.0 {
                    state.screen_width - config::POPUP_WIDTH - 10.0
                } else {
                    x.max(10.0)
                }
            };
            let target_y = config::BAR_HEIGHT;

            let popup_settings = window::Settings {
                size: Size::new(config::POPUP_WIDTH, config::POPUP_MIN_HEIGHT),
                position: window::Position::Specific(Point::new(target_x, target_y)),
                decorations: false,
                transparent: true,
                level: window::Level::AlwaysOnTop,
                resizable: false,
                visible: true,
                exit_on_close_request: false,
                ..Default::default()
            };

            let (id, open_task) = window::open(popup_settings);
            state.windows.insert(id, WindowKind::Popup { kind });

            let mut tasks = vec![close_task, open_task.map(Message::PopupWindowOpened)];
            if matches!(kind, PopupKind::Network) {
                state.network_status = "Scanning...".to_string();
                tasks.push(Task::perform(
                    async {
                        tokio::task::spawn_blocking(network::sync_scan)
                            .await
                            .unwrap_or_default()
                    },
                    Message::NetworkScanResult,
                ));
            }
            if matches!(kind, PopupKind::Audio) {
                tasks.push(Task::perform(
                    async {
                        tokio::task::spawn_blocking(audio_control::scan_audio_devices)
                            .await
                            .unwrap_or_default()
                    },
                    |(out, in_)| Message::AudioScanResult(out, in_),
                ));
            }
            if matches!(kind, PopupKind::Battery) {
                tasks.push(Task::perform(
                    async {
                        tokio::task::spawn_blocking(battery_control::get_power_plans)
                            .await
                            .unwrap_or_default()
                    },
                    Message::PowerPlanListResult,
                ));
            }
            if matches!(kind, PopupKind::Keyboard) {
                tasks.push(Task::perform(
                    async {
                        tokio::task::spawn_blocking(keyboard_control::get_all_layouts)
                            .await
                            .unwrap_or_default()
                    },
                    Message::KeyboardLayoutListResult,
                ));
            }
            Task::batch(tasks)
        }
        Message::ClosePopup => {
            close_all_popups(state)
        }
        Message::PopupWindowOpened(id) => {
            window::raw_id::<Message>(id).map(move |raw| Message::PopupHwndReady(id, raw))
        }
        Message::PopupHwndReady(id, raw_hwnd) => {
            win32::apply_popup_flags(raw_hwnd);
            let _ = id;
            Task::none()
        }
        Message::TrayIconClicked { id, action } => {
            state.tray_manager.send_action(&id, &action);
            Task::none()
        }
        Message::SettingsSpeakerVolume(val) => {
            audio_control::set_speaker_volume(val);
            state.speaker_volume = val;
            if val > 0.0 {
                state.speaker_muted = false;
            }
            Task::none()
        }
        Message::SettingsMicVolume(val) => {
            audio_control::set_mic_volume(val);
            state.mic_volume = val;
            Task::none()
        }
        Message::SettingsBrightness(val) => {
            brightness_control::set_brightness(val);
            state.brightness = val;
            Task::none()
        }
        Message::ToggleSpeakerMute => {
            audio_control::toggle_speaker_mute();
            state.speaker_muted = audio_control::is_speaker_muted();
            state.speaker_volume = audio_control::get_speaker_volume();
            Task::none()
        }
        Message::ToggleMicMute => {
            audio_control::toggle_mic_mute();
            state.mic_muted = audio_control::is_mic_muted();
            state.mic_volume = audio_control::get_mic_volume();
            Task::none()
        }
        Message::AudioScanDevices => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(audio_control::scan_audio_devices)
                        .await
                        .unwrap_or_default()
                },
                |(out, in_)| Message::AudioScanResult(out, in_),
            )
        }
        Message::AudioScanResult(outputs, inputs) => {
            state.output_devices = outputs;
            state.input_devices = inputs;
            state.current_output_device_id = audio_control::get_current_output_device_id();
            state.current_input_device_id = audio_control::get_current_input_device_id();
            Task::none()
        }
        Message::AudioSelectDevice { device_id, is_input } => {
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || audio_control::set_default_device(&device_id, is_input))
                        .await
                        .unwrap_or(false)
                },
                Message::AudioSelectDeviceResult,
            )
        }
        Message::AudioSelectDeviceResult(ok) => {
            if ok {
                state.current_output_device_id = audio_control::get_current_output_device_id();
                state.current_input_device_id = audio_control::get_current_input_device_id();
            }
            Task::none()
        }
        Message::MediaTogglePlay => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(audio_control::media_toggle_play_sync)
                        .await
                        .ok();
                    tokio::task::spawn_blocking(audio_control::get_media_state_sync)
                        .await
                        .unwrap_or_default()
                },
                Message::MediaStateResult,
            )
        }
        Message::MediaNextTrack => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(audio_control::media_next_track_sync)
                        .await
                        .ok();
                    tokio::task::spawn_blocking(audio_control::get_media_state_sync)
                        .await
                        .unwrap_or_default()
                },
                Message::MediaStateResult,
            )
        }
        Message::MediaPrevTrack => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(audio_control::media_prev_track_sync)
                        .await
                        .ok();
                    tokio::task::spawn_blocking(audio_control::get_media_state_sync)
                        .await
                        .unwrap_or_default()
                },
                Message::MediaStateResult,
            )
        }
        Message::MediaTick => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(audio_control::get_media_state_sync)
                        .await
                        .unwrap_or_default()
                },
                Message::MediaStateResult,
            )
        }
        Message::MediaStateResult(media) => {
            state.media_title = media.title;
            state.media_artist = media.artist;
            state.media_thumbnail = media.thumbnail;
            state.media_is_playing = media.is_playing;
            state.media_has_session = media.has_session;
            Task::none()
        }
        Message::ToggleWifi => {
            state.wifi_enabled = wireless_control::toggle_wifi();
            Task::none()
        }
        Message::ToggleBluetooth => {
            state.bluetooth_enabled = wireless_control::toggle_bluetooth();
            Task::none()
        }
        Message::ToggleAirplane => {
            state.airplane_enabled = wireless_control::toggle_airplane();
            Task::none()
        }
        Message::ToggleBatterySaver => {
            state.battery_saver_enabled = wireless_control::toggle_battery_saver();
            Task::none()
        }
        Message::PowerAction(action) => {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            match action {
                PowerAction::Lock => {
                    unsafe {
                        let _ = windows::Win32::System::Shutdown::LockWorkStation();
                    }
                }
                PowerAction::Sleep => {
                    let _ = Command::new("rundll32.exe")
                        .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                        .creation_flags(0x08000000)
                        .spawn();
                }
                PowerAction::Restart => {
                    let _ = Command::new("shutdown")
                        .args(["/r", "/t", "0"])
                        .creation_flags(0x08000000)
                        .spawn();
                }
                PowerAction::Shutdown => {
                    let _ = Command::new("shutdown")
                        .args(["/s", "/t", "0"])
                        .creation_flags(0x08000000)
                        .spawn();
                }
                PowerAction::Quit => {
                    return iced::exit();
                }
                PowerAction::OpenSettings => {
                    let _ = Command::new("cmd")
                        .args(["/c", "start", "ms-settings:"])
                        .creation_flags(0x08000000)
                        .spawn();
                }
            }
            Task::none()
        }
        Message::SyncSettings => {
            state.speaker_volume = audio_control::get_speaker_volume();
            state.mic_volume = audio_control::get_mic_volume();
            state.speaker_muted = audio_control::is_speaker_muted();
            state.mic_muted = audio_control::is_mic_muted();
            if let Some(b) = brightness_control::get_brightness() {
                state.brightness = b;
            }
            let wireless = wireless_control::get_state();
            state.wifi_enabled = wireless.wifi_enabled;
            state.bluetooth_enabled = wireless.bluetooth_enabled;
            state.airplane_enabled = wireless.airplane_enabled;
            state.battery_saver_enabled = wireless.battery_saver_enabled;
            Task::none()
        }
        Message::NetworkStatusTick => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(network::check_internet)
                        .await
                        .unwrap_or(false)
                },
                Message::NetworkStatusResult,
            )
        }
        Message::NetworkStatusResult(is_online) => {
            state.is_online = is_online;
            Task::none()
        }
        Message::NetworkScan => {
            state.network_status = "Scanning...".to_string();
            Task::perform(
                async {
                    tokio::task::spawn_blocking(network::sync_scan)
                        .await
                        .unwrap_or_default()
                },
                Message::NetworkScanResult,
            )
        }
        Message::NetworkScanResult(networks) => {
            state.networks = networks;
            state.network_status = "Ready".to_string();
            state.wifi_password_input = String::new();
            Task::none()
        }
        Message::NetworkPasswordChanged(pw) => {
            state.wifi_password_input = pw;
            Task::none()
        }
        Message::NetworkToggleExpand(ssid) => {
            if state.expanded_network.as_deref() == Some(&ssid) {
                state.expanded_network = None;
            } else {
                state.expanded_network = Some(ssid);
            }
            Task::none()
        }
        Message::NetworkConnect { ssid, password } => {
            state.network_status = format!("Connecting to {}...", ssid);
            let ssid_clone = ssid.clone();
            Task::perform(
                async move {
                    let ssid_for_block = ssid_clone.clone();
                    let ok = tokio::task::spawn_blocking(move || {
                        let pw = if password.is_empty() {
                            None
                        } else {
                            Some(password.as_str())
                        };
                        network::sync_connect(&ssid_for_block, pw)
                    })
                    .await
                    .unwrap_or(false);
                    (ssid_clone, ok)
                },
                |(ssid, ok)| Message::NetworkConnectResult { ssid, ok },
            )
        }
        Message::NetworkConnectResult { ssid, ok } => {
            if ok {
                state.network_status = "Connected".to_string();
                state.wifi_password_input = String::new();
                return Task::perform(
                    async {
                        tokio::task::spawn_blocking(network::sync_scan)
                            .await
                            .unwrap_or_default()
                    },
                    Message::NetworkScanResult,
                );
            } else {
                state.network_status = format!("Failed to connect to {}", ssid);
            }
            Task::none()
        }
        Message::NetworkDisconnect => {
            state.network_status = "Disconnecting...".to_string();
            Task::perform(
                async {
                    tokio::task::spawn_blocking(network::sync_disconnect)
                        .await
                        .unwrap_or(false)
                },
                Message::NetworkDisconnectResult,
            )
        }
        Message::NetworkDisconnectResult(ok) => {
            if ok {
                state.network_status = "Disconnected".to_string();
                return Task::perform(
                    async {
                        tokio::task::spawn_blocking(network::sync_scan)
                            .await
                            .unwrap_or_default()
                    },
                    Message::NetworkScanResult,
                );
            } else {
                state.network_status = "Disconnect failed".to_string();
            }
            Task::none()
        }
        Message::BatteryTick => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(battery_control::get_battery_info)
                        .await
                        .unwrap_or(None)
                },
                Message::BatteryPollResult,
            )
        }
        Message::BatteryPollResult(info) => {
            if let Some(info) = info {
                state.battery_percent = info.percent;
                state.battery_is_plugged = info.is_plugged;
                state.battery_secs_left = info.secs_left;
            }
            Task::none()
        }
        Message::PowerPlanList => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(battery_control::get_power_plans)
                        .await
                        .unwrap_or_default()
                },
                Message::PowerPlanListResult,
            )
        }
        Message::PowerPlanListResult(plans) => {
            state.battery_plans = plans;
            Task::none()
        }
        Message::PowerPlanSwitch(guid) => {
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || battery_control::set_power_plan(&guid))
                        .await
                        .ok();
                    tokio::task::spawn_blocking(battery_control::get_power_plans)
                        .await
                        .unwrap_or_default()
                },
                Message::PowerPlanListResult,
            )
        }
        Message::KeyboardTick => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(keyboard_control::get_active_layout)
                        .await
                        .unwrap_or(None)
                },
                Message::KeyboardPollResult,
            )
        }
        Message::KeyboardPollResult(layout) => {
            if let Some(layout) = layout {
                state.current_keyboard_hkl = Some(layout.hkl_raw);
                state.keyboard_lang_text = keyboard_control::get_bar_text(layout.hkl_raw);
            }
            Task::none()
        }
        Message::KeyboardLayoutList => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(keyboard_control::get_all_layouts)
                        .await
                        .unwrap_or_default()
                },
                Message::KeyboardLayoutListResult,
            )
        }
        Message::KeyboardLayoutListResult(layouts) => {
            state.keyboard_layouts = layouts;
            Task::none()
        }
        Message::KeyboardSwitchLayout(hkl_raw) => {
            let target = state.last_real_hwnd;
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        keyboard_control::switch_layout(target, hkl_raw);
                    })
                    .await
                    .ok();
                },
                |_| Message::KeyboardTick,
            )
        }
        Message::ProfileLoadResult(info) => {
            state.profile_display_name = info.display_name;
            state.profile_principal_name = info.principal_name;
            state.profile_avatar_handle = info.avatar.map(|(rgba, w, h)| {
                iced::widget::image::Handle::from_rgba(w, h, rgba)
            });
            state.profile_loaded = true;
            Task::none()
        }
        Message::ProfileOpenLauncher => {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            let _ = Command::new("cmd")
                .args(["/c", "start", ""])
                .creation_flags(0x08000000)
                .spawn();
            close_all_popups(state)
        }
        Message::UpdateCheckResult(info) => {
            state.update_info = *info;
            Task::none()
        }
        Message::UpdateDownloadStart => {
            if let Some(ref info) = state.update_info {
                state.update_downloading = true;
                state.update_download_progress = 0;
                let (tx, rx) = std::sync::mpsc::channel();
                state.update_rx = Some(Arc::new(std::sync::Mutex::new(rx)));
                let info_clone = info.clone();
                tokio::task::spawn_blocking(move || {
                    crate::updater::download_updates(&info_clone, tx);
                });
            }
            Task::none()
        }
        Message::UpdateApply => {
            if let Some(ref info) = state.update_info {
                let info_clone = info.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || crate::updater::apply_updates(&info_clone))
                            .await
                            .unwrap_or(false)
                    },
                    |_| Message::UpdateDismiss,
                )
            } else {
                Task::none()
            }
        }
        Message::UpdateDismiss => {
            state.update_info = None;
            state.update_ready = false;
            state.update_download_progress = 0;
            state.update_rx = None;
            Task::none()
        }
        Message::UpdateCheckAgain => {
            Task::perform(
                async {
                    tokio::task::spawn_blocking(crate::updater::check_for_updates)
                        .await
                        .unwrap_or(None)
                },
                |info| Message::UpdateCheckResult(Box::new(info)),
            )
        }
    }
}

fn close_all_popups(state: &mut State) -> Task<Message> {
    let popup_ids: Vec<WindowId> = state
        .windows
        .iter()
        .filter(|(_, kind)| matches!(kind, WindowKind::Popup { .. }))
        .map(|(id, _)| *id)
        .collect();

    let mut tasks = Vec::new();
    for id in popup_ids {
        tasks.push(window::close(id));
    }
    Task::batch(tasks)
}

pub fn view(state: &State, window_id: WindowId) -> Element<'_, Message> {
    if let Some(kind) = state.windows.get(&window_id) {
        match kind {
            WindowKind::Bar => {
                if state.initialized {
                    let tray_open = state.windows.values().any(|k| matches!(k, WindowKind::Popup { kind: PopupKind::Tray }));
                    iced::widget::mouse_area(bar_ui::bar(
                        &state.clock_text,
                        state.is_online,
                        tray_open,
                        state.speaker_volume,
                        state.speaker_muted,
                        state.battery_percent,
                        state.battery_is_plugged,
                        &state.keyboard_lang_text,
                        &state.profile_display_name,
                        state.update_info.is_some(),
                    ))
                        .on_enter(Message::BarMouseEnter)
                        .on_exit(Message::BarMouseExit)
                        .into()
                } else {
                    Element::from(iced::widget::text(""))
                }
            }
            WindowKind::Popup { kind } => {
                iced::widget::mouse_area(crate::popup::popup_view(
                    *kind,
                    &state.tray_manager,
                    state,
                ))
                .on_enter(Message::PopupMouseEnter)
                .on_exit(Message::PopupMouseExit)
                .into()
            }
        }
    } else {
        Element::from(iced::widget::text(""))
    }
}

pub fn subscription(state: &State) -> Subscription<Message> {
    let mut subs = vec![
        time::every(config::MONITOR_INTERVAL).map(|_| Message::MonitorTick),
        time::every(Duration::from_secs(1)).map(|_| {
            let now = chrono::Local::now();
            Message::ClockTick(now.format("%a %b %d   %I:%M %p").to_string())
        }),
        window::close_events().map(Message::WindowClosed),
        time::every(Duration::from_millis(500)).map(|_| Message::SyncSettings),
        time::every(config::AUDIO_POLL_RATE).map(|_| Message::MediaTick),
        time::every(Duration::from_secs(5)).map(|_| Message::NetworkStatusTick),
        time::every(Duration::from_secs(30)).map(|_| Message::BatteryTick),
        time::every(Duration::from_millis(500)).map(|_| Message::KeyboardTick),
    ];
    if state.slide.is_some() {
        subs.push(window::frames().map(Message::Frame));
    }
    Subscription::batch(subs)
}
