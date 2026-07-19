use iced::widget::{button, container, row, stack, text, Space};
use iced::{Color, Element, Fill, Length, Padding, Theme};

use crate::app::Message;
use crate::config;
use crate::popup::PopupKind;

pub fn bar(
    clock_text: &str,
    is_online: bool,
    tray_open: bool,
    speaker_volume: f32,
    speaker_muted: bool,
    battery_percent: u8,
    battery_is_plugged: bool,
    keyboard_lang: &str,
    profile_name: &str,
) -> Element<'static, Message> {
    let left = clickable_widget_str(lucide_icons::Icon::User, Some(profile_name), PopupKind::Profile);

    let lang = clickable_widget_str(lucide_icons::Icon::Languages, Some(keyboard_lang), PopupKind::Keyboard);
    let tray_icon = if tray_open {
        lucide_icons::Icon::ChevronUp
    } else {
        lucide_icons::Icon::ChevronDown
    };
    let tray = clickable_widget(tray_icon, None, PopupKind::Tray);

    let audio_icon = if speaker_muted || speaker_volume == 0.0 {
        lucide_icons::Icon::VolumeX
    } else if speaker_volume < 30.0 {
        lucide_icons::Icon::Volume1
    } else {
        lucide_icons::Icon::Volume2
    };
    let audio = clickable_widget(audio_icon, None, PopupKind::Audio);

    let net_icon = if is_online {
        lucide_icons::Icon::Wifi
    } else {
        lucide_icons::Icon::WifiOff
    };
    let net_label = if is_online { Some("Online") } else { Some("Offline") };
    let net = clickable_widget(net_icon, net_label, PopupKind::Network);

    let bat_icon = if battery_is_plugged {
        if battery_percent >= 95 { lucide_icons::Icon::BatteryFull }
        else if battery_percent >= 80 { lucide_icons::Icon::BatteryFull }
        else if battery_percent >= 60 { lucide_icons::Icon::BatteryMedium }
        else if battery_percent >= 40 { lucide_icons::Icon::BatteryMedium }
        else { lucide_icons::Icon::BatteryLow }
    } else {
        if battery_percent >= 95 { lucide_icons::Icon::BatteryFull }
        else if battery_percent >= 80 { lucide_icons::Icon::BatteryFull }
        else if battery_percent >= 60 { lucide_icons::Icon::BatteryMedium }
        else if battery_percent >= 40 { lucide_icons::Icon::BatteryMedium }
        else if battery_percent >= 20 { lucide_icons::Icon::BatteryLow }
        else { lucide_icons::Icon::BatteryWarning }
    };
    let bat = clickable_widget_str(bat_icon, Some(&format!("{}%", battery_percent)), PopupKind::Battery);
    let settings = clickable_widget(lucide_icons::Icon::Settings, None, PopupKind::Settings);

    let right = row![lang, tray, audio, net, bat, settings]
        .spacing(4)
        .align_y(iced::Alignment::Center);

    let bar_row = row![left, Space::new().width(Fill), right]
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([0, 20]));

    let bar_content = container(bar_row)
        .width(Length::Fill)
        .height(config::BAR_HEIGHT)
        .padding(Padding::from([0, config::FLOATING_MARGIN_X as u16]))
        .style(bar_container_style);

    let clock = text(clock_text.to_owned())
        .size(14)
        .color(Color::from_rgba(
            config::TEXT_COLOR[0],
            config::TEXT_COLOR[1],
            config::TEXT_COLOR[2],
            config::TEXT_COLOR[3],
        ))
        .center();

    let clock_container = container(clock)
        .width(300)
        .height(config::BAR_HEIGHT)
        .center_x(300)
        .center_y(config::BAR_HEIGHT);

    stack![bar_content]
        .push(
            container(clock_container)
                .width(Length::Fill)
                .height(config::BAR_HEIGHT)
                .center_x(Length::Fill)
                .center_y(config::BAR_HEIGHT),
        )
        .into()
}

fn bar_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            config::BG_COLOR[0],
            config::BG_COLOR[1],
            config::BG_COLOR[2],
            config::BG_COLOR[3],
        ))),
        border: iced::Border {
            color: Color::from_rgba(
                config::BORDER_COLOR[0],
                config::BORDER_COLOR[1],
                config::BORDER_COLOR[2],
                config::BORDER_COLOR[3],
            ),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn clickable_widget(
    icon: lucide_icons::Icon,
    label: Option<&'static str>,
    kind: PopupKind,
) -> Element<'static, Message> {
    let icon_char: char = icon.into();
    let icon_text = text(icon_char.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(Color::from_rgba(
            config::TEXT_COLOR[0],
            config::TEXT_COLOR[1],
            config::TEXT_COLOR[2],
            config::TEXT_COLOR[3],
        ));

    let content: Element<'static, Message> = if let Some(label) = label {
        let label_text = text(label)
            .size(13)
            .color(Color::from_rgba(
                config::TEXT_COLOR[0],
                config::TEXT_COLOR[1],
                config::TEXT_COLOR[2],
                config::TEXT_COLOR[3],
            ));
        row![icon_text, label_text]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .height(Fill)
            .into()
    } else {
        row![icon_text]
            .align_y(iced::Alignment::Center)
            .height(Fill)
            .into()
    };

    button(content)
        .padding(Padding::from([0, 8]))
        .height(config::BAR_HEIGHT)
        .style(|_theme: &Theme, status: button::Status| match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    config::HOVER_BG[0],
                    config::HOVER_BG[1],
                    config::HOVER_BG[2],
                    config::HOVER_BG[3],
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
            _ => button::Style {
                background: None,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
        })
        .on_press(Message::OpenPopup { kind })
        .into()
}

fn clickable_widget_str(
    icon: lucide_icons::Icon,
    label: Option<&str>,
    kind: PopupKind,
) -> Element<'static, Message> {
    let icon_char: char = icon.into();
    let icon_text = text(icon_char.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(Color::from_rgba(
            config::TEXT_COLOR[0],
            config::TEXT_COLOR[1],
            config::TEXT_COLOR[2],
            config::TEXT_COLOR[3],
        ));

    let content: Element<'static, Message> = if let Some(label) = label {
        let label_text = text(label.to_owned())
            .size(13)
            .color(Color::from_rgba(
                config::TEXT_COLOR[0],
                config::TEXT_COLOR[1],
                config::TEXT_COLOR[2],
                config::TEXT_COLOR[3],
            ));
        row![icon_text, label_text]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .height(Fill)
            .into()
    } else {
        row![icon_text]
            .align_y(iced::Alignment::Center)
            .height(Fill)
            .into()
    };

    button(content)
        .padding(Padding::from([0, 8]))
        .height(config::BAR_HEIGHT)
        .style(|_theme: &Theme, status: button::Status| match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    config::HOVER_BG[0],
                    config::HOVER_BG[1],
                    config::HOVER_BG[2],
                    config::HOVER_BG[3],
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
            _ => button::Style {
                background: None,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
        })
        .on_press(Message::OpenPopup { kind })
        .into()
}
