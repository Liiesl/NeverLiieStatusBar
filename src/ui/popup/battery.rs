use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{Color, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::config;

use super::widgets::*;

pub(crate) fn battery_popup_content(state: &crate::app::State) -> Element<'static, Message> {
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
    let header = row![icon_widget, info_col, Space::new().width(Length::Fill)]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    let progress_bg = Color::from_rgba(
        config::border_color()[0],
        config::border_color()[1],
        config::border_color()[2],
        config::border_color()[3],
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

            let mut row_content = row![plan_icon_widget, plan_name_text, Space::new().width(Length::Fill)]
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

    let settings_btn = button(text("Battery Settings").size(12))
        .padding(Padding::from([8, 10]))
        .width(Length::Fill)
        .on_press(Message::PowerAction(crate::app::PowerAction::OpenSettings))
        .style(|_theme: &Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(
                    config::hover_bg()[0],
                    config::hover_bg()[1],
                    config::hover_bg()[2],
                    config::hover_bg()[3],
                ),
                _ => Color::from_rgba(
                    config::border_color()[0],
                    config::border_color()[1],
                    config::border_color()[2],
                    config::border_color()[3],
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

    content.into()
}
