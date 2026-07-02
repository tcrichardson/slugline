use iced::widget::{container, row, text};
use iced::{Element, Length};

use slugline_core::doc::{Context, resolve_context, scan_document};
use slugline_core::editor::{EditorState, Mode};

use crate::app::Message;
use crate::theme_iced::Palette;

/// A one-line breadcrumb for where the cursor currently is, for the status line's
/// left segment (outside command mode). Port of `web/src/lib/components/StatusLine.svelte`'s
/// inline `context` derivation.
fn context_label(editor: &EditorState) -> String {
    let model = scan_document(&editor.lines);
    match resolve_context(&model, editor.cursor.line) {
        Context::None => String::new(),
        Context::Todo { .. } => "To Do".to_string(),
        Context::Meeting { block, .. } => format!("Meetings \u{203a} {}", block.name),
        Context::Note { block, .. } => format!("Notes \u{203a} {}", block.name),
        Context::Other { section } => section.title,
    }
}

/// The footer status line: mode + cursor-context breadcrumb on the left (or the typed
/// `:command` while command mode is active, matching the web's footer even though the
/// command palette overlay is the actual input surface now), the current editor message
/// right-aligned. Port of `web/src/lib/components/StatusLine.svelte`.
pub fn view<'a>(editor: &EditorState, palette: &Palette) -> Element<'a, Message> {
    let left: Element<'a, Message> = match &editor.command {
        Some(typed) => text(format!(":{typed}")).size(12).color(palette.fg).into(),
        None => {
            let mode_label = match editor.mode {
                Mode::Insert => "-- INSERT --",
                Mode::Normal => "-- NORMAL --",
            };
            row![
                text(mode_label).size(12).color(palette.fg),
                text(context_label(editor)).size(12).color(palette.muted),
            ]
            .spacing(16)
            .into()
        }
    };

    let status_bar = palette.status_bar;
    container(
        row![
            left,
            container(text(editor.message.clone()).size(12).color(palette.accent))
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(16)
        .width(Length::Fill),
    )
    .padding([4, 16])
    .width(Length::Fill)
    .style(move |_theme| iced::widget::container::Style {
        background: Some(status_bar.into()),
        ..iced::widget::container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slugline_core::editor::create_editor_state;

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn labels_the_to_do_section() {
        let raw = ["# T", "", "## To Do", "- [ ] x", "", "## Notes", ""];
        let mut editor = create_editor_state(lines(&raw), Vec::new());
        editor.cursor.line = 3;
        assert_eq!(context_label(&editor), "To Do");
    }

    #[test]
    fn labels_a_meeting_block_with_its_name() {
        let raw = [
            "# T",
            "",
            "## Meetings",
            "### Sync",
            "body",
            "",
            "## Notes",
            "",
        ];
        let mut editor = create_editor_state(lines(&raw), Vec::new());
        editor.cursor.line = 4;
        assert_eq!(context_label(&editor), "Meetings \u{203a} Sync");
    }

    #[test]
    fn is_empty_on_the_title_line() {
        let raw = ["# T", "", "## Notes", ""];
        let editor = create_editor_state(lines(&raw), Vec::new());
        assert_eq!(context_label(&editor), "");
    }
}
