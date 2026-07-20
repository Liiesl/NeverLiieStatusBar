#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod measure;
mod platform;
mod services;
mod ui;

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
        Some(app::WindowKind::Settings) => "StatusBar Settings".to_string(),
        None => String::new(),
    }
}

fn view(state: &app::State, window: iced::window::Id) -> iced::Element<'_, app::Message> {
    app::view(state, window)
}
