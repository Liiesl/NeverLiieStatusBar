use iced::widget::{button, column, container, row, rule, slider, text, Space};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::config;

pub(crate) const ACCENT: [f32; 4] = [96.0 / 255.0, 205.0 / 255.0, 1.0, 1.0];
pub(crate) const TILE_BG: [f32; 4] = [62.0 / 255.0, 62.0 / 255.0, 62.0 / 255.0, 1.0];
pub(crate) const TILE_HOVER: [f32; 4] = [78.0 / 255.0, 78.0 / 255.0, 78.0 / 255.0, 1.0];
pub(crate) const TEXT_SUB: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

pub(crate) fn text_color() -> Color {
    Color::from_rgba(
        config::TEXT_COLOR[0],
        config::TEXT_COLOR[1],
        config::TEXT_COLOR[2],
        config::TEXT_COLOR[3],
    )
}

pub(crate) fn sub_text_color() -> Color {
    Color::from_rgba(TEXT_SUB[0], TEXT_SUB[1], TEXT_SUB[2], TEXT_SUB[3])
}

pub(crate) fn accent_color() -> Color {
    Color::from_rgba(ACCENT[0], ACCENT[1], ACCENT[2], ACCENT[3])
}

pub(crate) fn section_label(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(12)
        .color(sub_text_color())
        .into()
}

pub(crate) fn tile_button(
    icon_char: char,
    label: &'static str,
    sub: String,
    active: bool,
    msg: Message,
) -> Element<'static, Message> {
    let icon = text(icon_char.to_string())
        .size(16)
        .font(iced::Font::with_name("lucide"))
        .color(if active { Color::BLACK } else { text_color() });

    let title = text(label)
        .size(11)
        .color(if active { Color::BLACK } else { text_color() });

    let subtitle = text(sub)
        .size(10)
        .color(if active {
            Color::from_rgba(0.1, 0.1, 0.1, 0.8)
        } else {
            sub_text_color()
        });

    let content = column![icon, title, subtitle].spacing(2);

    button(content)
        .padding(Padding::from([8, 10]))
        .width(Length::Fill)
        .height(52)
        .on_press(msg)
        .style(move |_theme: &Theme, status: button::Status| {
            let bg = if active {
                Color::from_rgba(ACCENT[0], ACCENT[1], ACCENT[2], ACCENT[3])
            } else {
                match status {
                    button::Status::Hovered => Color::from_rgba(
                        TILE_HOVER[0],
                        TILE_HOVER[1],
                        TILE_HOVER[2],
                        TILE_HOVER[3],
                    ),
                    _ => Color::from_rgba(TILE_BG[0], TILE_BG[1], TILE_BG[2], TILE_BG[3]),
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: if active {
                        accent_color()
                    } else {
                        Color::from_rgba(0.33, 0.33, 0.33, 1.0)
                    },
                },
                text_color: if active { Color::BLACK } else { text_color() },
                ..Default::default()
            }
        })
        .into()
}

pub(crate) fn modern_slider(
    icon_char: char,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'static,
    on_mute: Option<Message>,
) -> Element<'static, Message> {
    let icon = text(icon_char.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(sub_text_color());

    let icon_btn: Element<'static, Message> = if let Some(msg) = on_mute {
        button(icon)
            .padding(4)
            .on_press(msg)
            .style(|_theme: &Theme, status: button::Status| match status {
                button::Status::Hovered => button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
                _ => button::Style {
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
    } else {
        container(icon)
            .padding(4)
            .into()
    };

    let sl = slider(0.0..=100.0, value, on_change)
        .height(4)
        .style(|_theme: &Theme, status: slider::Status| {
            let handle_color = Color::WHITE;
            let rail_active = accent_color();
            let rail_inactive = Color::from_rgba(0.53, 0.53, 0.53, 1.0);

            let (rail_bg, handle_bg) = match status {
                slider::Status::Active => (rail_active, handle_color),
                slider::Status::Hovered => (rail_active, handle_color),
                slider::Status::Dragged => (rail_active, handle_color),
            };

            slider::Style {
                rail: slider::Rail {
                    backgrounds: (rail_bg.into(), rail_inactive.into()),
                    width: 4.0,
                    border: iced::Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                },
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 8.0 },
                    background: handle_bg.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        });

    row![icon_btn, sl].spacing(8).align_y(iced::Alignment::Center).into()
}

pub(crate) fn action_button(
    icon: lucide_icons::Icon,
    label: &'static str,
    msg: Message,
) -> Element<'static, Message> {
    let icon_char: char = icon.into();
    let icon_widget = text(icon_char.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(text_color());

    let label_text = text(label.to_string())
        .size(14)
        .color(text_color());

    let content = row![icon_widget, label_text]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    button(content)
        .padding(Padding::from([10, 10]))
        .width(Length::Fill)
        .on_press(msg)
        .style(|_theme: &Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(
                    TILE_HOVER[0], TILE_HOVER[1], TILE_HOVER[2], TILE_HOVER[3],
                ),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: text_color(),
                ..Default::default()
            }
        })
        .into()
}

pub(crate) fn initials_avatar(name: &str, size: f32) -> Element<'static, Message> {
    let initials = compute_initials(name);
    let border_radius = size / 2.0;

    let initial_text = text(initials)
        .size(size * 0.45)
        .color(Color::BLACK)
        .center()
        .width(Length::Fill);

    container(initial_text)
        .width(size)
        .height(size)
        .center_x(size)
        .center_y(size)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(accent_color())),
            border: iced::Border {
                radius: border_radius.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub(crate) fn compute_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() >= 2 {
        let first = parts[0].chars().next().unwrap_or('?');
        let second = parts[1].chars().next().unwrap_or('?');
        format!("{}{}", first, second).to_uppercase()
    } else if parts.len() == 1 && !parts[0].is_empty() {
        let chars: Vec<char> = parts[0].chars().take(2).collect();
        let s: String = chars.into_iter().collect();
        s.to_uppercase()
    } else {
        "??".to_string()
    }
}

pub(crate) fn signal_icon(bars: u8) -> lucide_icons::Icon {
    match bars {
        0 | 1 => lucide_icons::Icon::WifiZero,
        2 => lucide_icons::Icon::WifiLow,
        3 => lucide_icons::Icon::Wifi,
        4 => lucide_icons::Icon::WifiHigh,
        _ => lucide_icons::Icon::WifiHigh,
    }
}

pub(crate) fn volume_icon_char(volume: f32, muted: bool) -> lucide_icons::Icon {
    if muted || volume == 0.0 {
        lucide_icons::Icon::VolumeX
    } else if volume < 30.0 {
        lucide_icons::Icon::Volume1
    } else {
        lucide_icons::Icon::Volume2
    }
}

pub(crate) fn device_icon_char(name: &str, is_input: bool) -> lucide_icons::Icon {
    if is_input {
        return lucide_icons::Icon::Mic;
    }
    let lower = name.to_lowercase();
    if lower.contains("headphone") || lower.contains("headset") || lower.contains("buds") || lower.contains("airpods") {
        lucide_icons::Icon::Headphones
    } else if lower.contains("monitor") || lower.contains("tv") {
        lucide_icons::Icon::Monitor
    } else {
        lucide_icons::Icon::Speaker
    }
}

pub(crate) fn device_list_item(
    name: String,
    device_id: String,
    is_active: bool,
    is_input: bool,
) -> Element<'static, Message> {
    let icon_char: char = device_icon_char(&name, is_input).into();
    let icon = text(icon_char.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(if is_active { accent_color() } else { text_color() });

    let label = text(name)
        .size(12)
        .color(if is_active { text_color() } else { sub_text_color() });

    let mut row_content = row![icon, label]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    if is_active {
        let check_char: char = lucide_icons::Icon::Check.into();
        let check = text(check_char.to_string())
            .size(12)
            .font(iced::Font::with_name("lucide"))
            .color(accent_color());
        row_content = row_content.push(check);
    }

    let id_for_click = device_id.clone();
    button(row_content)
        .padding(Padding::from([8, 10]))
        .width(Length::Fill)
        .on_press(Message::AudioSelectDevice {
            device_id: id_for_click,
            is_input,
        })
        .style(move |_theme: &Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(0.25, 0.25, 0.25, 1.0),
                _ => Color::from_rgba(0.15, 0.15, 0.15, 1.0),
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: if is_active {
                        accent_color()
                    } else {
                        Color::from_rgba(0.3, 0.3, 0.3, 1.0)
                    },
                },
                text_color: text_color(),
                ..Default::default()
            }
        })
        .into()
}

pub(crate) fn media_control_card(state: &crate::app::State) -> Element<'static, Message> {
    let title_text = if state.media_title.is_empty() {
        "No Media".to_string()
    } else if state.media_title.len() > 30 {
        format!("{}...", &state.media_title[..27])
    } else {
        state.media_title.clone()
    };

    let artist_text = if state.media_artist.len() > 30 {
        format!("{}...", &state.media_artist[..27])
    } else {
        state.media_artist.clone()
    };

    let art: Element<'static, Message> = if !state.media_thumbnail.is_empty() {
        container(text("♪".to_string()).size(20).color(sub_text_color()))
            .width(48)
            .height(48)
            .center_x(48)
            .center_y(48)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 1.0))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    } else {
        container(text("♪".to_string()).size(20).color(sub_text_color()))
            .width(48)
            .height(48)
            .center_x(48)
            .center_y(48)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 1.0))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    };

    let title = text(title_text)
        .size(13)
        .color(text_color());

    let artist = text(artist_text)
        .size(11)
        .color(sub_text_color());

    let text_col = column![title, artist].spacing(2);

    let info_row = row![art, text_col, Space::new().width(Length::Fill)]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    fn transport_btn(icon: lucide_icons::Icon, msg: Message, size: f32) -> Element<'static, Message> {
        let icon_char: char = icon.into();
        let c = text(icon_char.to_string())
            .size(size)
            .font(iced::Font::with_name("lucide"))
            .color(text_color());

        button(c)
            .padding(6)
            .on_press(msg)
            .style(|_theme: &Theme, status: button::Status| match status {
                button::Status::Hovered => button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
                _ => button::Style {
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

    let play_icon = if state.media_is_playing {
        lucide_icons::Icon::Pause
    } else {
        lucide_icons::Icon::Play
    };

    let controls = row![
        transport_btn(lucide_icons::Icon::SkipBack, Message::MediaPrevTrack, 14.0),
        transport_btn(play_icon, Message::MediaTogglePlay, 16.0),
        transport_btn(lucide_icons::Icon::SkipForward, Message::MediaNextTrack, 14.0),
    ]
    .spacing(15)
    .align_y(iced::Alignment::Center);

    let controls_row = row![Space::new().width(Length::Fill), controls, Space::new().width(Length::Fill)];

    let content = column![info_row, Space::new().height(4), controls_row]
        .spacing(4);

    container(content)
        .padding(Padding::from([10, 10]))
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 1.0))),
            border: iced::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgba(0.3, 0.3, 0.3, 1.0),
            },
            ..Default::default()
        })
        .into()
}

pub(crate) fn divider() -> Element<'static, Message> {
    rule::horizontal(1).style(|_theme: &Theme| rule::Style {
        color: Color::from_rgba(
            config::BORDER_COLOR[0],
            config::BORDER_COLOR[1],
            config::BORDER_COLOR[2],
            config::BORDER_COLOR[3],
        ),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }).into()
}

pub(crate) fn popup_inner_style(_theme: &Theme) -> container::Style {
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
