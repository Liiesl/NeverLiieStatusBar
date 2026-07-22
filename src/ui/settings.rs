use iced::widget::{button, column, container, row, scrollable, slider, text, Space};
use iced::{Color, Element, Length, Padding, Theme};
use neverliie_iced_widgets::slider_tooltip::SliderTooltip;

use crate::app::{Message, SettingsPage};
use crate::config;
use crate::ui::popup::widgets::{
    accent_color, divider, popup_inner_style, section_label, sub_text_color, text_color,
};

pub fn settings_view(state: &crate::app::State) -> Element<'static, Message> {
    let header = text("StatusBar Settings")
        .size(20)
        .color(text_color());

    let version = text(env!("CARGO_PKG_VERSION"))
        .size(12)
        .color(sub_text_color());

    let header_row = row![header, Space::new().width(Length::Fill), version]
        .align_y(iced::Alignment::Center);

    let tabs = tab_bar(state.settings_page);

    let content: Element<'static, Message> = match state.settings_page {
        SettingsPage::General => general_page(),
        SettingsPage::Appearance => appearance_page(),
        SettingsPage::About => about_page(),
    };

    let footer = text(format!("Config: {}", config::settings_file()))
        .size(11)
        .color(sub_text_color());

    let layout = column![
        header_row,
        divider(),
        tabs,
        divider(),
        content,
        Space::new().height(Length::Fill),
        divider(),
        footer,
    ]
    .spacing(8)
    .padding(Padding::from([16, 20]));

    container(scrollable(layout))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(popup_inner_style)
        .into()
}

fn tab_bar(active: SettingsPage) -> Element<'static, Message> {
    let general = tab_button("General", active == SettingsPage::General, SettingsPage::General);
    let appearance = tab_button(
        "Behavior",
        active == SettingsPage::Appearance,
        SettingsPage::Appearance,
    );
    let about = tab_button("About", active == SettingsPage::About, SettingsPage::About);

    row![general, appearance, about]
        .spacing(4)
        .into()
}

fn tab_button(label: &str, active: bool, page: SettingsPage) -> Element<'static, Message> {
    let label_owned = label.to_owned();
    button(
        text(label_owned)
            .size(13)
            .color(if active {
                Color::WHITE
            } else {
                sub_text_color()
            }),
    )
    .padding(Padding::from([6, 16]))
    .on_press(Message::SettingsPageSelected(page))
    .style(move |_theme: &Theme, status: button::Status| {
        let bg = if active {
            accent_color()
        } else {
            match status {
                button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                _ => Color::TRANSPARENT,
            }
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            text_color: if active {
                Color::WHITE
            } else {
                text_color()
            },
            ..Default::default()
        }
    })
    .into()
}

fn general_page() -> Element<'static, Message> {
    let section = section_label("Behavior");

    let auto_hide = settings_slider(
        "Auto-hide delay",
        config::auto_hide_delay().as_millis() as f32,
        100.0..=3000.0,
        "ms",
        Message::SettingsSetAutoHideDelay,
    );

    let trigger_dwell = settings_slider(
        "Trigger dwell time",
        config::trigger_dwell_time().as_millis() as f32,
        100.0..=2000.0,
        "ms",
        Message::SettingsSetTriggerDwell,
    );

    let anim_duration = settings_slider(
        "Animation duration",
        config::anim_duration().as_millis() as f32,
        0.0..=1000.0,
        "ms",
        Message::SettingsSetAnimDuration,
    );

    column![section, auto_hide, trigger_dwell, anim_duration]
        .spacing(12)
        .into()
}

fn appearance_page() -> Element<'static, Message> {
    let section = section_label("Trigger Zone");

    let trigger_height = settings_slider(
        "Mouse trigger height",
        config::mouse_trigger_height(),
        1.0..=20.0,
        "px",
        Message::SettingsSetMouseTriggerHeight,
    );

    column![section, trigger_height].spacing(12).into()
}

fn about_page() -> Element<'static, Message> {
    let app_name = text("NeverLiieStatusBar")
        .size(18)
        .color(text_color());

    let version = text(format!("Version {}", env!("CARGO_PKG_VERSION")))
        .size(13)
        .color(sub_text_color());

    let description = text("A lightweight, auto-hiding status bar for Windows.")
        .size(13)
        .color(sub_text_color());

    let config_label = section_label("Configuration");

    let config_path = text(config::settings_file())
        .size(12)
        .color(sub_text_color());

    let open_folder = button(
        text("Open Config Folder")
            .size(13)
            .color(text_color()),
    )
    .padding(Padding::from([8, 16]))
    .on_press(Message::SettingsOpenConfigFolder)
    .style(|_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.05),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba(0.33, 0.33, 0.33, 1.0),
            },
            text_color: text_color(),
            ..Default::default()
        }
    });

    column![
        app_name,
        version,
        Space::new().height(8),
        description,
        Space::new().height(24),
        config_label,
        config_path,
        Space::new().height(8),
        open_folder,
    ]
    .spacing(4)
    .into()
}

fn settings_slider(
    label: &str,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    on_change: impl Fn(f32) -> Message + 'static,
) -> Element<'static, Message> {
    let title = text(label.to_owned())
        .size(13)
        .color(text_color());

    let value_display = format!("{:.0} {}", value, suffix);
    let value_text = text(value_display)
        .size(12)
        .color(sub_text_color())
        .width(Length::Fixed(70.0))
        .align_x(iced::Alignment::End);

    let sl = SliderTooltip::new(range, value, on_change)
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

    column![
        title,
        row![sl, value_text]
            .spacing(8)
            .align_y(iced::Alignment::Center),
    ]
    .spacing(4)
    .into()
}
