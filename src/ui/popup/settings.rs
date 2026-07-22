use iced::widget::{column, row, Space};
use iced::{Background, Border, Color, Element, Length, Shadow};

use neverliie_iced_widgets::confirmation_dialog::{ButtonStyle, ConfirmationDialog, Style};

use crate::app::{Message, PowerAction};
use crate::config;

use super::widgets::*;

fn dialog_style(_theme: &iced::Theme) -> Style {
    let bg = config::bg_color();
    let border = config::border_color();
    let text = config::text_color();

    Style {
        backdrop_color: Color { a: 0.5, ..Color::from_rgba(bg[0], bg[1], bg[2], bg[3]) },
        background: Background::Color(Color::from_rgba(bg[0], bg[1], bg[2], bg[3])),
        border: Border {
            width: 1.0,
            radius: 8.0.into(),
            color: Color::from_rgba(border[0], border[1], border[2], border[3]),
        },
        shadow: Shadow::default(),
        title_color: Color::from_rgba(text[0], text[1], text[2], text[3]),
        message_color: Color::from_rgba(text[0], text[1], text[2], text[3]).scale_alpha(0.7),
        secondary_button_background: Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 1.0)),
        secondary_button_border: Border {
            width: 1.0,
            radius: 4.0.into(),
            color: Color::from_rgba(border[0], border[1], border[2], border[3]),
        },
        secondary_button_text_color: Color::from_rgba(text[0], text[1], text[2], text[3]),
        button_background: Background::Color(accent_color()),
        button_border: Border {
            width: 0.0,
            radius: 4.0.into(),
            color: Color::TRANSPARENT,
        },
        button_text_color: Color::WHITE,
        danger_button_background: Background::Color(Color::from_rgb(0.7, 0.2, 0.2)),
        danger_button_border: Border {
            width: 0.0,
            radius: 4.0.into(),
            color: Color::TRANSPARENT,
        },
        danger_button_text_color: Color::WHITE,
    }
}

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
        power_btn(lucide_icons::Icon::Lock, Message::ShowPowerConfirm(PowerAction::Lock)),
        power_btn(lucide_icons::Icon::Moon, Message::ShowPowerConfirm(PowerAction::Sleep)),
        power_btn(lucide_icons::Icon::RotateCcw, Message::ShowPowerConfirm(PowerAction::Restart)),
        power_btn(lucide_icons::Icon::Power, Message::ShowPowerConfirm(PowerAction::Shutdown)),
        power_btn(lucide_icons::Icon::EyeOff, Message::ShowPowerConfirm(PowerAction::Quit)),
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

    let (dialog_title, dialog_message, blocking) = match state.pending_power_action {
        Some(PowerAction::Lock) => ("Lock PC?", "Your PC will be locked.", false),
        Some(PowerAction::Sleep) => ("Sleep?", "Your PC will go to sleep.", false),
        Some(PowerAction::Restart) => (
            "Restart PC?",
            "Your PC will restart immediately. Any unsaved work will be lost.",
            true,
        ),
        Some(PowerAction::Shutdown) => (
            "Shut Down PC?",
            "Your PC will shut down immediately. Any unsaved work will be lost.",
            true,
        ),
        Some(PowerAction::Quit) => ("Quit App?", "NeverLiie StatusBar will close.", false),
        _ => return content.into(),
    };

    let mut dialog = ConfirmationDialog::new(content, true, dialog_title, dialog_message)
        .on_confirm(Message::PowerConfirm)
        .on_cancel(Message::PowerCancel)
        .on_dismiss(Message::PowerCancel)
        .no_pointer()
        .style(dialog_style);

    if blocking {
        dialog = dialog.blocking();
    }

    if matches!(state.pending_power_action, Some(PowerAction::Restart) | Some(PowerAction::Shutdown)) {
        dialog = dialog.button(
            neverliie_iced_widgets::confirmation_dialog::DialogButton::new("Confirm", Message::PowerConfirm)
                .style(ButtonStyle::Danger),
        );
    }

    dialog.into()
}
