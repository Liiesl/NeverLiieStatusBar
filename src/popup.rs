use iced::widget::{button, column, container, row, rule, scrollable, slider, stack, text, text_input, Space};
use iced::{Color, Element, Fill, Length, Padding, Theme};

use crate::app::{Message, PowerAction};
use crate::config;
use crate::network;
use crate::systray::{SystemTrayManager, TrayIconAction};

const ACCENT: [f32; 4] = [96.0 / 255.0, 205.0 / 255.0, 255.0 / 255.0, 1.0];
const TILE_BG: [f32; 4] = [62.0 / 255.0, 62.0 / 255.0, 62.0 / 255.0, 1.0];
const TILE_HOVER: [f32; 4] = [78.0 / 255.0, 78.0 / 255.0, 78.0 / 255.0, 1.0];
const TEXT_SUB: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    Profile,
    Battery,
    Network,
    Audio,
    Keyboard,
    Tray,
    Settings,
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
            config::TEXT_COLOR[0],
            config::TEXT_COLOR[1],
            config::TEXT_COLOR[2],
            config::TEXT_COLOR[3],
        ))
        .center()
        .width(Length::Fill);

    let divider = rule::horizontal(1).style(|_theme: &Theme| rule::Style {
        color: Color::from_rgba(
            config::BORDER_COLOR[0],
            config::BORDER_COLOR[1],
            config::BORDER_COLOR[2],
            config::BORDER_COLOR[3],
        ),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    });

    let body: Element<'static, Message> = match kind {
        PopupKind::Tray => tray_popup_content(tray_manager),
        PopupKind::Settings => settings_popup_content(state),
        PopupKind::Network => network_popup_content(state),
        PopupKind::Audio => audio_popup_content(state),
        PopupKind::Battery => battery_popup_content(state),
        PopupKind::Keyboard => keyboard_popup_content(state),
        PopupKind::Profile => profile_popup_content(state),
    };

    let content = container(
        column![title, divider, body]
            .spacing(12)
            .padding(Padding::from([16.0, 16.0])),
    )
    .width(config::POPUP_WIDTH)
    .height(Length::Fill)
    .style(popup_inner_style);

    content.into()
}

fn text_color() -> Color {
    Color::from_rgba(
        config::TEXT_COLOR[0],
        config::TEXT_COLOR[1],
        config::TEXT_COLOR[2],
        config::TEXT_COLOR[3],
    )
}

fn sub_text_color() -> Color {
    Color::from_rgba(TEXT_SUB[0], TEXT_SUB[1], TEXT_SUB[2], TEXT_SUB[3])
}

fn accent_color() -> Color {
    Color::from_rgba(ACCENT[0], ACCENT[1], ACCENT[2], ACCENT[3])
}

fn section_label(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(12)
        .color(sub_text_color())
        .into()
}

fn tile_button(
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

fn modern_slider(
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

fn settings_popup_content(state: &crate::app::State) -> Element<'static, Message> {
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

    let power_label = text("Power")
        .size(13)
        .color(text_color());

    fn power_btn(icon: lucide_icons::Icon, msg: Message) -> Element<'static, Message> {
        let icon_char: char = icon.into();
        let c = text(icon_char.to_string())
            .size(14)
            .font(iced::Font::with_name("lucide"))
            .color(text_color());

        let content = row![c]
            .align_y(iced::Alignment::Center)
            .height(Fill);

        button(content)
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

    let power_row = row![
        power_btn(lucide_icons::Icon::Lock, Message::PowerAction(PowerAction::Lock)),
        power_btn(lucide_icons::Icon::Moon, Message::PowerAction(PowerAction::Sleep)),
        power_btn(lucide_icons::Icon::RotateCcw, Message::PowerAction(PowerAction::Restart)),
        power_btn(lucide_icons::Icon::Power, Message::PowerAction(PowerAction::Shutdown)),
        power_btn(lucide_icons::Icon::EyeOff, Message::PowerAction(PowerAction::Quit)),
        Space::new().width(Fill),
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

fn network_popup_content(state: &crate::app::State) -> Element<'static, Message> {
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

fn signal_icon(bars: u8) -> lucide_icons::Icon {
    match bars {
        0 | 1 => lucide_icons::Icon::WifiZero,
        2 => lucide_icons::Icon::WifiLow,
        3 => lucide_icons::Icon::Wifi,
        4 => lucide_icons::Icon::WifiHigh,
        _ => lucide_icons::Icon::WifiHigh,
    }
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

    let mut header_row = row![sig_widget, ssid_text, Space::new().width(Fill)]
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
                    radius: if expanded { 6.0 } else { 6.0 }.into(),
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

fn tray_popup_content(tray_manager: &SystemTrayManager) -> Element<'static, Message> {
    let icons = tray_manager.icons();

    if icons.is_empty() {
        return text("No tray icons")
            .size(12)
            .color(Color::from_rgba(0.6, 0.6, 0.6, 1.0))
            .center()
            .width(Length::Fill)
            .into();
    }

    let mut icon_list = column![].spacing(2);

    for icon in icons.values() {
        let icon_id = icon.id.clone();
        let tooltip = if icon.tooltip.is_empty() {
            "Unknown".to_string()
        } else {
            icon.tooltip.clone()
        };

        // Icon image (if available)
        let icon_widget: Element<'static, Message> = if let Some(handle) = &icon.cached_image_handle {
            iced::widget::image(handle.clone())
                .width(24)
                .height(24)
                .into()
        } else {
            // Fallback: small colored circle
            container(text(""))
                .width(24)
                .height(24)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.4, 0.4, 0.4, 1.0))),
                    border: iced::Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        };

        let label = text(tooltip)
            .size(13)
            .color(Color::from_rgba(
                config::TEXT_COLOR[0],
                config::TEXT_COLOR[1],
                config::TEXT_COLOR[2],
                config::TEXT_COLOR[3],
            ));

        let row_content = row![icon_widget, label]
            .spacing(10)
            .align_y(iced::Alignment::Center);

        let id_clone = icon_id.clone();
        let id_right = icon_id.clone();
        let id_middle = icon_id.clone();
        let row_button = button(row_content)
            .padding(Padding::from([6, 10]))
            .width(Length::Fill)
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
            .on_press(Message::TrayIconClicked {
                id: id_clone,
                action: TrayIconAction::LeftClick,
            });

        let row_button = iced::widget::mouse_area(row_button)
            .on_right_press(Message::TrayIconClicked {
                id: id_right,
                action: TrayIconAction::RightClick,
            })
            .on_middle_press(Message::TrayIconClicked {
                id: id_middle,
                action: TrayIconAction::MiddleClick,
            });

        icon_list = icon_list.push(row_button);
    }

    scrollable(icon_list.padding(4)).into()
}

// ===========================================================================
// Audio Popup
// ===========================================================================

fn volume_icon_char(volume: f32, muted: bool) -> lucide_icons::Icon {
    if muted || volume == 0.0 {
        lucide_icons::Icon::VolumeX
    } else if volume < 30.0 {
        lucide_icons::Icon::Volume1
    } else if volume < 70.0 {
        lucide_icons::Icon::Volume2
    } else {
        lucide_icons::Icon::Volume2
    }
}

fn device_icon_char(name: &str, is_input: bool) -> lucide_icons::Icon {
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

fn device_list_item(
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

fn media_control_card(state: &crate::app::State) -> Element<'static, Message> {
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

    // Album art placeholder
    let art: Element<'static, Message> = if !state.media_thumbnail.is_empty() {
        // For now, show a placeholder since iced image handling is complex
        // In future: iced::widget::image with bytes
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

    let info_row = row![art, text_col, Space::new().width(Fill)]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    // Transport controls
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

    let controls_row = row![Space::new().width(Fill), controls, Space::new().width(Fill)];

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

fn audio_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let spk_icon = volume_icon_char(state.speaker_volume, state.speaker_muted);
    let mic_icon_char: char = (if state.mic_muted {
        lucide_icons::Icon::MicOff
    } else {
        lucide_icons::Icon::Mic
    }).into();

    let sliders = column![
        section_label("Master Volume"),
        modern_slider(
            spk_icon.into(),
            state.speaker_volume,
            Message::SettingsSpeakerVolume,
            Some(Message::ToggleSpeakerMute),
        ),
        modern_slider(
            mic_icon_char.into(),
            state.mic_volume,
            Message::SettingsMicVolume,
            Some(Message::ToggleMicMute),
        ),
    ]
    .spacing(4);

    // Output devices
    let mut output_list = column![].spacing(2);
    let current_out = state.current_output_device_id.as_deref().unwrap_or("");
    for dev in &state.output_devices {
        let is_active = dev.id == current_out;
        output_list = output_list.push(device_list_item(
            dev.name.clone(),
            dev.id.clone(),
            is_active,
            false,
        ));
    }
    if state.output_devices.is_empty() {
        output_list = output_list.push(
            text("No output devices found")
                .size(11)
                .color(sub_text_color())
        );
    }

    let output_section = column![
        section_label("Output Device"),
        output_list,
    ]
    .spacing(4);

    // Input devices
    let mut input_list = column![].spacing(2);
    let current_in = state.current_input_device_id.as_deref().unwrap_or("");
    for dev in &state.input_devices {
        let is_active = dev.id == current_in;
        input_list = input_list.push(device_list_item(
            dev.name.clone(),
            dev.id.clone(),
            is_active,
            true,
        ));
    }
    if state.input_devices.is_empty() {
        input_list = input_list.push(
            text("No input devices found")
                .size(11)
                .color(sub_text_color())
        );
    }

    let input_section = column![
        section_label("Input Device"),
        input_list,
    ]
    .spacing(4);

    let mut content = column![sliders, divider(), output_section, divider(), input_section]
        .spacing(12);

    // Media player section (if available)
    if state.media_has_session || !state.media_title.is_empty() {
        let media_section = column![
            section_label("Media Player"),
            media_control_card(state),
        ]
        .spacing(4);
        content = content.push(divider()).push(media_section);
    }

    scrollable(content.padding(4)).into()
}

// ===========================================================================
// Profile Popup
// ===========================================================================

fn profile_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let display_name = &state.profile_display_name;
    let principal_name = &state.profile_principal_name;

    // --- Avatar ---
    let avatar_size: f32 = 100.0;
    let avatar: Element<'static, Message> =
        if let Some(handle) = &state.profile_avatar_handle {
            iced::widget::image(handle.clone())
                .width(avatar_size)
                .height(avatar_size)
                .into()
        } else {
            initials_avatar(display_name, avatar_size)
        };

    let avatar_container = container(avatar)
        .width(avatar_size)
        .height(avatar_size)
        .center_x(avatar_size)
        .center_y(avatar_size)
        .clip(true)
        .style(|_theme: &Theme| container::Style {
            border: iced::Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // --- Name ---
    let name_text = text(display_name.clone())
        .size(18)
        .color(text_color())
        .center()
        .width(Length::Fill);

    // --- Principal name ---
    let email_text = text(principal_name.clone())
        .size(13)
        .color(sub_text_color())
        .center()
        .width(Length::Fill);

    let user_info = column![avatar_container, name_text, email_text]
        .spacing(8)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill);

    // --- Divider ---
    let divider_line = divider();

    // --- Launcher status ---
    let launcher_online = false; // TODO: implement IPC ping when available
    let dot_color = if launcher_online {
        Color::from_rgba(0.30, 0.69, 0.31, 1.0) // green
    } else {
        Color::from_rgba(0.96, 0.26, 0.21, 1.0) // red
    };
    let status_text_str = if launcher_online { "Launcher Online" } else { "Launcher Offline" };

    let dot = text("●".to_string())
        .size(18)
        .color(dot_color);

    let status_label = text(status_text_str.to_string())
        .size(13)
        .color(text_color());

    let status_row = row![dot, status_label, Space::new().width(Fill)]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    let status_container = container(status_row)
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                TILE_BG[0], TILE_BG[1], TILE_BG[2], TILE_BG[3],
            ))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // --- Action buttons ---
    let open_settings_btn = action_button(
        lucide_icons::Icon::Settings,
        "  Account Settings",
        Message::PowerAction(PowerAction::OpenSettings),
    );

    let system_settings_btn = action_button(
        lucide_icons::Icon::Cog,
        "  Open Settings",
        Message::PowerAction(PowerAction::OpenSettings),
    );

    let buttons = column![open_settings_btn, system_settings_btn]
        .spacing(5)
        .width(Length::Fill);

    let content = column![
        user_info,
        divider_line,
        status_container,
        buttons,
    ]
    .spacing(15)
    .width(Length::Fill);

    scrollable(content.padding(4)).into()
}

fn initials_avatar(name: &str, size: f32) -> Element<'static, Message> {
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

fn compute_initials(name: &str) -> String {
    let parts: Vec<&str> = name.trim().split_whitespace().collect();
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

fn action_button(
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

// ===========================================================================
// Battery Popup
// ===========================================================================

fn battery_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let percent = state.battery_percent;
    let is_plugged = state.battery_is_plugged;
    let secs_left = state.battery_secs_left;

    let status_text = if is_plugged {
        if percent >= 99 {
            "Fully Charged".to_string()
        } else {
            "Charging".to_string()
        }
    } else if secs_left == -2 {
        "On Battery".to_string()
    } else if secs_left == -1 {
        "Estimating...".to_string()
    } else {
        let hrs = secs_left / 3600;
        let mins = (secs_left % 3600) / 60;
        format!("{} hr {} min remaining", hrs, mins)
    };

    let percent_text = text(format!("{}%", percent))
        .size(20)
        .color(text_color());

    let status_label = text(status_text)
        .size(12)
        .color(sub_text_color());

    let bat_icon: char = if is_plugged {
        lucide_icons::Icon::BatteryCharging.into()
    } else if percent > 60 {
        lucide_icons::Icon::BatteryFull.into()
    } else if percent > 30 {
        lucide_icons::Icon::BatteryMedium.into()
    } else if percent > 15 {
        lucide_icons::Icon::BatteryLow.into()
    } else {
        lucide_icons::Icon::BatteryWarning.into()
    };

    let icon_color = if !is_plugged && percent < 20 {
        Color::from_rgba(1.0, 0.27, 0.27, 1.0)
    } else if is_plugged {
        accent_color()
    } else {
        text_color()
    };

    let icon_widget = text(bat_icon.to_string())
        .size(24)
        .font(iced::Font::with_name("lucide"))
        .color(icon_color);

    let info_col = column![percent_text, status_label].spacing(2);
    let header = row![icon_widget, info_col, Space::new().width(Fill)]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    // Progress bar
    let progress_bg = Color::from_rgba(
        config::BORDER_COLOR[0],
        config::BORDER_COLOR[1],
        config::BORDER_COLOR[2],
        config::BORDER_COLOR[3],
    );
    let progress_fill = if !is_plugged && percent < 20 {
        Color::from_rgba(1.0, 0.27, 0.27, 1.0)
    } else {
        accent_color()
    };
    let progress_bar = container(text(""))
        .width(Length::Fill)
        .height(4)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(progress_bg)),
            border: iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let progress_fill_bar = container(text(""))
        .width(Length::Fixed(percent as f32 * 2.8))
        .height(4)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(progress_fill)),
            border: iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let progress_stack = stack![progress_bar]
        .push(progress_fill_bar);

    // Power plans section
    let plans_label = section_label("Power Mode");

    let mut plans_list = column![].spacing(2);
    if state.battery_plans.is_empty() {
        plans_list = plans_list.push(
            text("No power plans available")
                .size(11)
                .color(sub_text_color()),
        );
    } else {
        for plan in &state.battery_plans {
            let is_active = plan.is_active;
            let plan_name = plan.name.clone();
            let plan_guid = plan.guid.clone();

            let plan_icon: char = if plan_name.to_lowercase().contains("balanced") {
                lucide_icons::Icon::Scale.into()
            } else if plan_name.to_lowercase().contains("save")
                || plan_name.to_lowercase().contains("eco")
            {
                lucide_icons::Icon::Leaf.into()
            } else {
                lucide_icons::Icon::Zap.into()
            };

            let plan_icon_widget = text(plan_icon.to_string())
                .size(14)
                .font(iced::Font::with_name("lucide"))
                .color(if is_active { accent_color() } else { text_color() });

            let plan_name_text = text(plan_name)
                .size(12)
                .color(if is_active { text_color() } else { sub_text_color() });

            let mut row_content = row![plan_icon_widget, plan_name_text, Space::new().width(Fill)]
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

            let guid_clone = plan_guid.clone();
            let plan_btn = button(row_content)
                .padding(Padding::from([8, 10]))
                .width(Length::Fill)
                .on_press(Message::PowerPlanSwitch(guid_clone))
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
                });

            plans_list = plans_list.push(plan_btn);
        }
    }

    // Settings button
    let settings_btn = button(text("Battery Settings").size(12))
        .padding(Padding::from([8, 10]))
        .width(Length::Fill)
        .on_press(Message::PowerAction(crate::app::PowerAction::OpenSettings))
        .style(|_theme: &Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(
                    config::HOVER_BG[0],
                    config::HOVER_BG[1],
                    config::HOVER_BG[2],
                    config::HOVER_BG[3],
                ),
                _ => Color::from_rgba(
                    config::BORDER_COLOR[0],
                    config::BORDER_COLOR[1],
                    config::BORDER_COLOR[2],
                    config::BORDER_COLOR[3],
                ),
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: text_color(),
                ..Default::default()
            }
        });

    let content = column![
        header,
        progress_stack,
        plans_label,
        plans_list,
        settings_btn,
    ]
    .spacing(10);

    scrollable(content.padding(4)).into()
}

// ===========================================================================
// Keyboard Popup
// ===========================================================================

fn keyboard_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let current_hkl = state.current_keyboard_hkl;

    if state.keyboard_layouts.is_empty() {
        return column![
            text("No layouts found")
                .size(12)
                .color(sub_text_color())
                .center()
                .width(Length::Fill)
        ]
        .spacing(8)
        .into();
    }

    let mut layout_list = column![].spacing(2);

    for layout in &state.keyboard_layouts {
        let is_active = current_hkl == Some(layout.hkl_raw);
        let hkl_raw = layout.hkl_raw;
        let short = layout.short_name.clone();
        let display = layout.display_name.clone();

        let short_text = text(format!("  {}  ", short))
            .size(12)
            .color(if is_active { Color::BLACK } else { text_color() });

        let display_text = text(display)
            .size(12)
            .color(if is_active { Color::BLACK } else { sub_text_color() });

        let row_content = row![short_text, display_text]
            .spacing(8)
            .align_y(iced::Alignment::Center);

        let layout_btn = button(row_content)
            .padding(Padding::from([8, 10]))
            .width(Length::Fill)
            .on_press(Message::KeyboardSwitchLayout(hkl_raw))
            .style(move |_theme: &Theme, status: button::Status| {
                let (bg, border_color) = if is_active {
                    (accent_color(), accent_color())
                } else {
                    match status {
                        button::Status::Hovered => (
                            Color::from_rgba(0.25, 0.25, 0.25, 1.0),
                            Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                        ),
                        _ => (
                            Color::from_rgba(0.15, 0.15, 0.15, 1.0),
                            Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                        ),
                    }
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: border_color,
                    },
                    text_color: if is_active { Color::BLACK } else { text_color() },
                    ..Default::default()
                }
            });

        layout_list = layout_list.push(layout_btn);
    }

    scrollable(layout_list.padding(4)).into()
}

fn divider() -> Element<'static, Message> {
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

fn popup_inner_style(_theme: &Theme) -> container::Style {
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
