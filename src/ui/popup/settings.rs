use iced::widget::{column, row, scrollable, Space};
use iced::{Element, Length};

use crate::app::{Message, PowerAction};

use super::widgets::*;

pub(crate) fn settings_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let wifi_sub = if state.wifi_enabled { "On".to_string() } else { "Off".to_string() };
    let bt_sub = if state.bluetooth_enabled { "On".to_string() } else { "Off".to_string() };
    let ap_sub = if state.airplane_enabled { "On".to_string() } else { "Off".to_string() };
    let bs_sub = if state.battery_saver_enabled { "On".to_string() } else { "Off".to_string() };

    let tile_wifi = tile_button(
        lucide_icons::Icon::Wifi.into(),
        "Wi-Fi",
        wifi_sub,
        state.wifi_enabled,
        Message::ToggleWifi,
    );
    let tile_bt = tile_button(
        lucide_icons::Icon::Bluetooth.into(),
        "Bluetooth",
        bt_sub,
        state.bluetooth_enabled,
        Message::ToggleBluetooth,
    );
    let tile_airplane = tile_button(
        lucide_icons::Icon::Plane.into(),
        "Airplane",
        ap_sub,
        state.airplane_enabled,
        Message::ToggleAirplane,
    );
    let tile_saver = tile_button(
        lucide_icons::Icon::Leaf.into(),
        "Battery Saver",
        bs_sub,
        state.battery_saver_enabled,
        Message::ToggleBatterySaver,
    );

    let grid = column![
        row![tile_wifi, tile_bt].spacing(8),
        row![tile_airplane, tile_saver].spacing(8),
    ]
    .spacing(8);

    let spk_icon = if state.speaker_muted {
        lucide_icons::Icon::VolumeX
    } else {
        lucide_icons::Icon::Volume2
    };
    let mic_icon = if state.mic_muted {
        lucide_icons::Icon::MicOff
    } else {
        lucide_icons::Icon::Mic
    };

    let sliders = column![
        section_label("Brightness"),
        modern_slider(
            lucide_icons::Icon::Sun.into(),
            state.brightness,
            Message::SettingsBrightness,
            None,
        ),
        Space::new().height(6),
        section_label("Audio"),
        modern_slider(
            spk_icon.into(),
            state.speaker_volume,
            Message::SettingsSpeakerVolume,
            Some(Message::ToggleSpeakerMute),
        ),
        modern_slider(
            mic_icon.into(),
            state.mic_volume,
            Message::SettingsMicVolume,
            Some(Message::ToggleMicMute),
        ),
    ]
    .spacing(4);

    let power_label = iced::widget::text("Power")
        .size(13)
        .color(text_color());

    fn power_btn(icon: lucide_icons::Icon, msg: Message) -> Element<'static, Message> {
        let icon_char: char = icon.into();
        let c = iced::widget::text(icon_char.to_string())
            .size(14)
            .font(iced::Font::with_name("lucide"))
            .color(text_color());

        let content = row![c]
            .align_y(iced::Alignment::Center);

        iced::widget::button(content)
            .padding(6)
            .on_press(msg)
            .style(|_theme: &iced::Theme, status: iced::widget::button::Status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
                _ => iced::widget::button::Style {
                    background: None,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
            })
            .into()
    }

    let power_row = row![
        power_btn(lucide_icons::Icon::Lock, Message::PowerAction(PowerAction::Lock)),
        power_btn(lucide_icons::Icon::Moon, Message::PowerAction(PowerAction::Sleep)),
        power_btn(lucide_icons::Icon::RotateCcw, Message::PowerAction(PowerAction::Restart)),
        power_btn(lucide_icons::Icon::Power, Message::PowerAction(PowerAction::Shutdown)),
        power_btn(lucide_icons::Icon::EyeOff, Message::PowerAction(PowerAction::Quit)),
        Space::new().width(Length::Fill),
        power_btn(
            lucide_icons::Icon::Settings,
            Message::PowerAction(PowerAction::OpenSettings),
        ),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let power_section = column![power_label, power_row].spacing(6);

    let content = column![grid, sliders, power_section].spacing(16);
    scrollable(content).into()
}
