use iced::widget::{column, text};
use iced::Element;

use crate::app::Message;

use super::widgets::*;

pub(crate) fn audio_popup_content(state: &crate::app::State) -> Element<'static, Message> {
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
            mic_icon_char,
            state.mic_volume,
            Message::SettingsMicVolume,
            Some(Message::ToggleMicMute),
        ),
    ]
    .spacing(4);

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

    if !state.media_players.is_empty() {
        let mut media_list = column![].spacing(6);
        for (i, player) in state.media_players.iter().enumerate() {
            let thumb_handle = state.media_player_thumbnail_handles.get(i).and_then(|h| h.as_ref());
            media_list = media_list.push(media_control_card(player, thumb_handle, i));
        }
        let media_section = column![
            section_label("Media Players"),
            media_list,
        ]
        .spacing(4);
        content = content.push(divider()).push(media_section);
    }

    content.into()
}
