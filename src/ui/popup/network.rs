use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::services::network;

use super::widgets::*;

pub(crate) fn network_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let status_str = state.network_status.clone();
    let status_label = text(status_str)
        .size(12)
        .color(sub_text_color())
        .center()
        .width(Length::Fill);

    let refresh_btn = button(text("Refresh").size(12))
        .padding(Padding::from([6, 12]))
        .on_press(Message::NetworkScan)
        .style(|_theme: &Theme, status: button::Status| match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.3, 0.3, 0.3, 1.0,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: text_color(),
                ..Default::default()
            },
            _ => button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.2, 0.2, 0.2, 1.0,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: text_color(),
                ..Default::default()
            },
        });

    let header = row![status_label, refresh_btn]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    if state.networks.is_empty() {
        return column![
            header,
            text("No networks found")
                .size(12)
                .color(sub_text_color())
                .center()
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .spacing(8)
        .into();
    }

    let mut network_list = column![].spacing(2);

    for net in &state.networks {
        let is_expanded = state.expanded_network.as_deref() == Some(&net.ssid);
        network_list = network_list.push(network_item(net, &state.wifi_password_input, is_expanded));
    }

    column![header, scrollable(network_list.padding(4))].into()
}

fn network_item(net: &network::NetworkInfo, password_value: &str, is_expanded: bool) -> Element<'static, Message> {
    let is_connected = net.is_connected;
    let expanded = is_expanded || is_connected;

    let sig_icon: char = signal_icon(net.signal_bars).into();
    let sig_color = if is_connected { accent_color() } else { text_color() };
    let sig_widget = text(sig_icon.to_string())
        .size(14)
        .font(iced::Font::with_name("lucide"))
        .color(sig_color);

    let ssid_text = text(net.ssid.clone())
        .size(13)
        .color(text_color());

    let mut header_row = row![sig_widget, ssid_text, Space::new().width(Length::Fill)]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    if net.is_secure {
        let lock_icon: char = lucide_icons::Icon::Lock.into();
        let lock = text(lock_icon.to_string())
            .size(12)
            .font(iced::Font::with_name("lucide"))
            .color(sub_text_color());
        header_row = header_row.push(lock);
    }

    if is_connected {
        let connected_label = text("Connected")
            .size(10)
            .color(accent_color());
        header_row = header_row.push(connected_label);
    } else if net.has_saved_profile {
        let saved_label = text("Saved")
            .size(10)
            .color(sub_text_color());
        header_row = header_row.push(saved_label);
    }

    let ssid_for_toggle = net.ssid.clone();
    let header_btn = button(header_row)
        .padding(Padding::from([8, 10]))
        .width(Length::Fill)
        .on_press(Message::NetworkToggleExpand(ssid_for_toggle))
        .style(move |_theme: &Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(0.2, 0.2, 0.2, 1.0),
                _ => Color::from_rgba(0.12, 0.12, 0.12, 1.0),
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: if is_connected {
                        accent_color()
                    } else {
                        Color::from_rgba(0.3, 0.3, 0.3, 1.0)
                    },
                },
                text_color: text_color(),
                ..Default::default()
            }
        });

    if !expanded {
        return header_btn.into();
    }

    let detail_content: Element<'static, Message> = if is_connected {
        button(text("Disconnect").size(11))
            .padding(Padding::from([6, 12]))
            .width(Length::Fill)
            .on_press(Message::NetworkDisconnect)
            .style(|_theme: &Theme, status: button::Status| match status {
                button::Status::Hovered => button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.8, 0.2, 0.2, 0.3,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
                _ => button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.3, 0.3, 0.3, 1.0,
                    ))),
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
        let needs_password = net.is_secure && !net.has_saved_profile;
        let ssid_clone = net.ssid.clone();
        let mut items = column![].spacing(6);

        if needs_password {
            let pw_input = text_input("Password", password_value)
                .secure(true)
                .padding(Padding::from([6, 8]))
                .width(Length::Fill)
                .on_input(Message::NetworkPasswordChanged);
            items = items.push(pw_input);
        }

        let pw = if needs_password {
            password_value.to_string()
        } else {
            String::new()
        };

        let connect_btn = button(text("Connect").size(11))
            .padding(Padding::from([6, 12]))
            .width(Length::Fill)
            .on_press(Message::NetworkConnect {
                ssid: ssid_clone,
                password: pw,
            })
            .style(|_theme: &Theme, status: button::Status| match status {
                button::Status::Hovered => button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.2, 0.5, 0.8, 0.3,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
                _ => button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.3, 0.3, 0.3, 1.0,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: text_color(),
                    ..Default::default()
                },
            });
        items = items.push(connect_btn);
        items.into()
    };

    let content = column![header_btn, detail_content].spacing(0);

    container(content)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 1.0))),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: if is_connected {
                    accent_color()
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 1.0)
                },
            },
            ..Default::default()
        })
        .into()
}
