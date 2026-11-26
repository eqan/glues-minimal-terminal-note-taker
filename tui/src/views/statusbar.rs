use {
    crate::{
        context::{NotebookContext, notebook::ContextState},
        logger::*,
        theme::THEME,
    },
    glues_core::state::State,
    ratatui::{
        Frame,
        layout::{
            Constraint::{Length, Percentage},
            Layout, Rect,
        },
        style::Stylize,
        text::{Line, Span, Text},
    },
};

pub fn draw(frame: &mut Frame, area: Rect, state: &State, context: &NotebookContext) {
    let description = format!(" {}", state.describe().log_unwrap());
    let insert_mode = matches!(context.state, ContextState::EditorInsertMode);
    let preview_mode = context.markdown_preview;
    let normal_editor = matches!(context.state, ContextState::EditorNormalMode { .. });
    let [desc_area, keymap_area] = Layout::horizontal([Percentage(100), Length(36)]).areas(area);

    frame.render_widget(
        Text::raw(description)
            .fg(THEME.inactive_text)
            .bg(THEME.panel),
        desc_area,
    );

    frame.render_widget(
        Line::from(vec![
            Span::raw("").fg(THEME.success).bg(THEME.panel),
            Span::raw(if preview_mode {
                if context.wrap_content {
                    " [F2] Wrap on · [Ctrl+P] Edit "
                } else {
                    " [F2] Wrap off · [Ctrl+P] Edit "
                }
            } else if normal_editor {
                " [Ctrl+O / Ctrl+\\] Pane · [Ctrl+P] Preview "
            } else if insert_mode {
                " [Ctrl+O / Ctrl+\\] Pane · [Ctrl+H] Keymap "
            } else {
                " [Ctrl+O / Ctrl+\\] Pane · [?] Keymap "
            })
            .fg(THEME.success_text)
            .bg(THEME.success),
        ]),
        keymap_area,
    );
}
