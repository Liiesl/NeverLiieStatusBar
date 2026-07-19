#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio_control;
mod bar_ui;
mod battery_control;
mod brightness_control;
mod config;
mod icon_utils;
mod ipc;
mod keyboard_control;
mod network;
mod popup;
mod profile_control;
mod systray;
mod updater;
mod wireless_control;
mod win32;

use iced::daemon;
use lucide_icons::LUCIDE_FONT_BYTES;
use velopack::VelopackApp;

fn main() -> iced::Result {
    VelopackApp::build().run();

    daemon(
        app::boot,
        app::update,
        view,
    )
    .title(title)
    .subscription(app::subscription)
    .font(LUCIDE_FONT_BYTES)
    .run()
}

fn title(state: &app::State, window: iced::window::Id) -> String {
    match state.windows.get(&window) {
        Some(app::WindowKind::Bar) => "NeverLiieStatusBar".to_string(),
        Some(app::WindowKind::Popup { kind }) => kind.title().to_string(),
        None => String::new(),
    }
}

fn view(state: &app::State, window: iced::window::Id) -> iced::Element<'_, app::Message> {
    app::view(state, window)
}
