use iced::widget::{button, container, row, stack, text, Space};
use iced::{Color, Element, Fill, Length, Padding, Theme};

use crate::app::Message;
use crate::config;
use crate::ui::popup::PopupKind;

#[allow(clippy::too_many_arguments)]
pub fn bar(
    clock_text: &str,
    is_online: bool,
    tray_open: bool,
    speaker_volume: f32,
    speaker_muted: bool,
    battery_percent: u8,
    _battery_is_plugged: bool,
    keyboard_lang: &str,
    profile_name: &str,
    has_update: bool,
) -> Element<'static, Message> {
    let profile = clickable_widget_str(lucide_icons::Icon::User, Some(profile_name), PopupKind::Profile);

    let left: Element<'static, Message> = if has_update {
        let update_widget = clickable_widget_accent(lucide_icons::Icon::ArrowUpCircle, Some("Update available"), PopupKind::Update);
        row![profile, update_widget]
            .spacing(0)
            .align_y(iced::Alignment::Center)
            .into()
    } else {
        profile
    };

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

    let bat_icon = if battery_percent >= 80 { lucide_icons::Icon::BatteryFull }
        else if battery_percent >= 40 { lucide_icons::Icon::BatteryMedium }
        else if battery_percent >= 20 { lucide_icons::Icon::BatteryLow }
        else { lucide_icons::Icon::BatteryWarning };
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
        .height(config::bar_height())
        .padding(Padding::from([0, config::floating_margin_x() as u16]))
        .style(bar_container_style);

    let clock = text(clock_text.to_owned())
        .size(14)
        .color(Color::from_rgba(
            config::text_color()[0],
            config::text_color()[1],
            config::text_color()[2],
            config::text_color()[3],
        ))
        .center();

    let clock_container = container(clock)
        .width(300)
        .height(config::bar_height())
        .center_x(300)
        .center_y(config::bar_height());

    stack![bar_content]
        .push(
            container(clock_container)
                .width(Length::Fill)
                .height(config::bar_height())
                .center_x(Length::Fill)
                .center_y(config::bar_height()),
        )
        .into()
}

fn bar_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            config::bg_color()[0],
            config::bg_color()[1],
            config::bg_color()[2],
            config::bg_color()[3],
        ))),
        border: iced::Border {
            color: Color::from_rgba(
                config::border_color()[0],
                config::border_color()[1],
                config::border_color()[2],
                config::border_color()[3],
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
            config::text_color()[0],
            config::text_color()[1],
            config::text_color()[2],
            config::text_color()[3],
        ));

    let content: Element<'static, Message> = if let Some(label) = label {
        let label_text = text(label)
            .size(13)
            .color(Color::from_rgba(
                config::text_color()[0],
                config::text_color()[1],
                config::text_color()[2],
                config::text_color()[3],
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
        .height(config::bar_height())
        .style(|_theme: &Theme, status: button::Status| match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    config::hover_bg()[0],
                    config::hover_bg()[1],
                    config::hover_bg()[2],
                    config::hover_bg()[3],
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
            config::text_color()[0],
            config::text_color()[1],
            config::text_color()[2],
            config::text_color()[3],
        ));

    let content: Element<'static, Message> = if let Some(label) = label {
        let label_text = text(label.to_owned())
            .size(13)
            .color(Color::from_rgba(
                config::text_color()[0],
                config::text_color()[1],
                config::text_color()[2],
                config::text_color()[3],
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
        .height(config::bar_height())
        .style(|_theme: &Theme, status: button::Status| match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    config::hover_bg()[0],
                    config::hover_bg()[1],
                    config::hover_bg()[2],
                    config::hover_bg()[3],
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

fn clickable_widget_accent(
    icon: lucide_icons::Icon,
    label: Option<&str>,
    kind: PopupKind,
) -> Element<'static, Message> {
    let accent = Color::from_rgba(96.0 / 255.0, 205.0 / 255.0, 1.0, 1.0);

    let icon_char: char = icon.into();
    let icon_text = text(icon_char.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(accent);

    let content: Element<'static, Message> = if let Some(label) = label {
        let label_text = text(label.to_owned())
            .size(13)
            .color(accent);
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
        .height(config::bar_height())
        .style(|_theme: &Theme, status: button::Status| match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    config::hover_bg()[0],
                    config::hover_bg()[1],
                    config::hover_bg()[2],
                    config::hover_bg()[3],
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
