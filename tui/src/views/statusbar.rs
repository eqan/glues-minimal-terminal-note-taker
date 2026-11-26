use {
    crate::{
        context::{NotebookContext, notebook::ContextState},
        theme::THEME,
    },
    ratatui::{
        style::Stylize,
        text::{Line, Span},
    },
};

pub fn controls(context: &NotebookContext) -> Line<'static> {
    let editor_action = if context.markdown_preview {
        "Edit"
    } else {
        "Preview"
    };
    let paste =
        if !context.markdown_preview && matches!(context.state, ContextState::EditorInsertMode) {
            " · Ctrl+V Paste"
        } else {
            ""
        };
    Line::from(vec![
        Span::raw("").fg(THEME.success).bg(THEME.panel),
        Span::raw(format!(" Ctrl+O/\\ Pane · Ctrl+P {editor_action}{paste} "))
            .fg(THEME.success_text)
            .bg(THEME.success),
    ])
    .right_aligned()
}
