use iced::widget::{button, column, row, scrollable, text};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::Message;

use super::widgets::*;

pub(crate) fn keyboard_popup_content(state: &crate::app::State) -> Element<'static, Message> {
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
