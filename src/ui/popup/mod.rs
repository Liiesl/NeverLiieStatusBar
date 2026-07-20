mod audio;
mod battery;
mod keyboard;
mod network;
mod profile;
mod settings;
mod tray;
pub(crate) mod widgets;
mod update;

use iced::widget::{column, container, rule, text};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::config;
use crate::platform::systray::SystemTrayManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    Profile,
    Battery,
    Network,
    Audio,
    Keyboard,
    Tray,
    Settings,
    Update,
}

impl PopupKind {
    pub fn title(self) -> &'static str {
        match self {
            PopupKind::Profile => "System",
            PopupKind::Battery => "Power",
            PopupKind::Network => "Wi-Fi",
            PopupKind::Audio => "Audio Mixer",
            PopupKind::Keyboard => "Keyboard Layout",
            PopupKind::Tray => "System Tray",
            PopupKind::Settings => "Quick Settings",
            PopupKind::Update => "App Update",
        }
    }
}

pub fn popup_view(
    kind: PopupKind,
    tray_manager: &SystemTrayManager,
    state: &crate::app::State,
) -> Element<'static, Message> {
    let title = text(kind.title())
        .size(14)
        .color(Color::from_rgba(
            config::text_color()[0],
            config::text_color()[1],
            config::text_color()[2],
            config::text_color()[3],
        ))
        .center()
        .width(Length::Fill);

    let divider = rule::horizontal(1).style(|_theme: &Theme| rule::Style {
        color: Color::from_rgba(
            config::border_color()[0],
            config::border_color()[1],
            config::border_color()[2],
            config::border_color()[3],
        ),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    });

    let body: Element<'static, Message> = match kind {
        PopupKind::Tray => tray::tray_popup_content(tray_manager),
        PopupKind::Settings => settings::settings_popup_content(state),
        PopupKind::Network => network::network_popup_content(state),
        PopupKind::Audio => audio::audio_popup_content(state),
        PopupKind::Battery => battery::battery_popup_content(state),
        PopupKind::Keyboard => keyboard::keyboard_popup_content(state),
        PopupKind::Profile => profile::profile_popup_content(state),
        PopupKind::Update => update::update_popup_content(state),
    };

    let content = container(
        column![title, divider, body]
            .spacing(12)
            .padding(Padding::from([16.0, 16.0])),
    )
    .width(config::popup_width())
    .height(Length::Fill)
    .style(widgets::popup_inner_style);

    content.into()
}
