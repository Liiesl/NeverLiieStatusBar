use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::config;
use crate::platform::systray::{SystemTrayManager, TrayIconAction};

use super::widgets::*;

pub(crate) fn tray_popup_content(tray_manager: &SystemTrayManager) -> Element<'static, Message> {
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

        let icon_widget: Element<'static, Message> = if let Some(handle) = &icon.cached_image_handle {
            iced::widget::image(handle.clone())
                .width(24)
                .height(24)
                .into()
        } else {
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
            .color(text_color());

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
