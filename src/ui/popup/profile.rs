use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::{Message, PowerAction};

use super::widgets::*;

pub(crate) fn profile_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let display_name = &state.profile_display_name;
    let principal_name = &state.profile_principal_name;

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

    let name_text = text(display_name.clone())
        .size(18)
        .color(text_color())
        .center()
        .width(Length::Fill);

    let email_text = text(principal_name.clone())
        .size(13)
        .color(sub_text_color())
        .center()
        .width(Length::Fill);

    let user_info = column![avatar_container, name_text, email_text]
        .spacing(8)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill);

    let divider_line = divider();

    let launcher_online = false;
    let dot_color = if launcher_online {
        Color::from_rgba(0.30, 0.69, 0.31, 1.0)
    } else {
        Color::from_rgba(0.96, 0.26, 0.21, 1.0)
    };
    let status_text_str = if launcher_online { "Launcher Online" } else { "Launcher Offline" };

    let dot = text("●".to_string())
        .size(18)
        .color(dot_color);

    let status_label = text(status_text_str.to_string())
        .size(13)
        .color(text_color());

    let status_row = row![dot, status_label, Space::new().width(Length::Fill)]
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
