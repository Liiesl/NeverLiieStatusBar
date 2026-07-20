use iced::widget::{column, row, text};
use iced::{Color, Element, Length, Theme};

use crate::app::Message;
use crate::config;

use super::widgets::*;

pub(crate) fn update_popup_content(state: &crate::app::State) -> Element<'static, Message> {
    let current_version = crate::services::updater::get_current_version();

    let version_label = |v: &str| -> Element<'static, Message> {
        text(format!("v{}", v))
            .size(12)
            .color(sub_text_color())
            .into()
    };

    if state.update_ready {
        let ready_icon = text(lucide_icons::Icon::CheckCircle.to_string())
            .size(24)
            .font(iced::Font::with_name("lucide"))
            .color(accent_color());

        let ready_title = text("Update Ready!")
            .size(14)
            .color(text_color());

        let ready_sub = text("The update has been downloaded and is ready to apply.")
            .size(12)
            .color(sub_text_color());

        let header = column![ready_icon, ready_title, ready_sub]
            .spacing(4)
            .align_x(iced::Alignment::Center);

        let restart_btn = action_button(
            lucide_icons::Icon::RotateCw,
            "Restart Now",
            Message::UpdateApply,
        );

        let later_btn = action_button(
            lucide_icons::Icon::Clock,
            "Later",
            Message::UpdateDismiss,
        );

        let body = column![header, divider(), restart_btn, later_btn]
            .spacing(10)
            .width(Length::Fill);

        return body.into();
    }

    if state.update_downloading {
        let dl_icon = text(lucide_icons::Icon::Download.to_string())
            .size(24)
            .font(iced::Font::with_name("lucide"))
            .color(accent_color());

        let pct = state.update_download_progress.clamp(0, 100) as u32;

        let dl_title = text(format!("Downloading v{}", state.update_info.as_ref().map(|i| i.TargetFullRelease.Version.as_str()).unwrap_or("...")))
            .size(14)
            .color(text_color());

        let pct_text = text(format!("{}%", pct))
            .size(12)
            .color(sub_text_color());

        let progress_bg = Color::from_rgba(
            config::border_color()[0],
            config::border_color()[1],
            config::border_color()[2],
            config::border_color()[3],
        );
        let progress_fill = accent_color();

        let bar_width = Length::Fill;
        let progress_bar = iced::widget::container(
            iced::widget::container(text(""))
                .width(Length::FillPortion(pct as u16))
                .height(4)
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(progress_fill)),
                    border: iced::Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .width(bar_width)
        .height(4)
        .style(move |_theme: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(progress_bg)),
            border: iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let notice = text("Please don't close the application.")
            .size(12)
            .color(sub_text_color());

        let header = column![dl_icon, dl_title, pct_text, progress_bar, notice]
            .spacing(8)
            .align_x(iced::Alignment::Center);

        let body = column![header]
            .spacing(10)
            .width(Length::Fill);

        return body.into();
    }

    if let Some(ref info) = state.update_info {
        let new_version = &info.TargetFullRelease.Version;

        let up_icon = text(lucide_icons::Icon::ArrowUpCircle.to_string())
            .size(24)
            .font(iced::Font::with_name("lucide"))
            .color(accent_color());

        let up_title = text("Update Available")
            .size(14)
            .color(text_color());

        let version_row = row![
            version_label(&current_version),
            text(" → ").size(12).color(sub_text_color()),
            text(format!("v{}", new_version)).size(12).color(accent_color()),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        let header = column![up_icon, up_title, version_row]
            .spacing(4)
            .align_x(iced::Alignment::Center);

        let mut sections = column![header].spacing(10).width(Length::Fill);

        if !info.TargetFullRelease.NotesMarkdown.is_empty() {
            let notes_label = text("Release Notes")
                .size(12)
                .color(sub_text_color());

            let notes_text = text(info.TargetFullRelease.NotesMarkdown.clone())
                .size(12)
                .color(text_color());

            sections = sections.push(divider());
            sections = sections.push(column![notes_label, notes_text].spacing(4));
        }

        let download_btn = action_button(
            lucide_icons::Icon::Download,
            "Download Update",
            Message::UpdateDownloadStart,
        );

        let dismiss_btn = action_button(
            lucide_icons::Icon::X,
            "Dismiss",
            Message::UpdateDismiss,
        );

        let body = sections.push(divider()).push(download_btn).push(dismiss_btn);

        body.into()
    } else {
        let check_icon = text(lucide_icons::Icon::CheckCircle.to_string())
            .size(24)
            .font(iced::Font::with_name("lucide"))
            .color(accent_color());

        let up_to_date = text("You're up to date!")
            .size(14)
            .color(text_color());

        let ver = text(format!("Current version: v{}", current_version))
            .size(12)
            .color(sub_text_color());

        let header = column![check_icon, up_to_date, ver]
            .spacing(4)
            .align_x(iced::Alignment::Center);

        let check_again = action_button(
            lucide_icons::Icon::RefreshCw,
            "Check Again",
            Message::UpdateCheckAgain,
        );

        let body = column![header, divider(), check_again]
            .spacing(10)
            .width(Length::Fill);

        body.into()
    }
}
