use {
    crate::{
        context::{NotebookContext, notebook::ContextState},
        theme::THEME,
    },
    ratatui::{
        Frame,
        layout::{Constraint, Layout, Rect},
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::Block,
    },
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    context: &NotebookContext,
    last_log: Option<&str>,
    dirty: bool,
) {
    if area.is_empty() {
        return;
    }

    frame.render_widget(Block::new().bg(THEME.panel), area);

    let (mode, mode_bg) = if context.markdown_preview {
        (
            Span::raw(" PREVIEW ")
                .fg(THEME.accent_text)
                .bg(THEME.accent),
            THEME.accent,
        )
    } else {
        match context.state {
            ContextState::EditorNormalMode { .. } => (
                Span::raw(" NORMAL ").fg(THEME.text).bg(THEME.surface),
                THEME.surface,
            ),
            ContextState::EditorInsertMode => (
                Span::raw(" INSERT ").fg(THEME.accent_text).bg(THEME.accent),
                THEME.accent,
            ),
            ContextState::EditorVisualMode => (
                Span::raw(" VISUAL ").fg(THEME.error_text).bg(THEME.error),
                THEME.error,
            ),
            _ => (
                Span::raw(" NOTEBOOK ")
                    .fg(THEME.text_secondary)
                    .bg(THEME.surface),
                THEME.surface,
            ),
        }
    };

    let wrap = if context.wrap_content { "on" } else { "off" };
    let mut left = vec![
        mode,
        Span::raw("").fg(mode_bg).bg(THEME.panel),
        Span::raw(format!(" [F2] Wrap {wrap} "))
            .fg(THEME.hint)
            .bg(THEME.panel),
    ];
    if dirty {
        left.extend([
            Span::raw("").fg(THEME.accent).bg(THEME.panel),
            Span::raw(" Saving... ")
                .fg(THEME.accent_text)
                .bg(THEME.accent),
        ]);
    } else if let Some(message) = last_log {
        left.extend([
            Span::raw("").fg(THEME.success).bg(THEME.panel),
            Span::raw(format!(" {message} "))
                .fg(THEME.success_text)
                .bg(THEME.success),
        ]);
    }

    let left = Line::from(left).style(Style::default().bg(THEME.panel));
    let right = controls(context);
    let right_width = right.width().min(area.width as usize) as u16;
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(right_width)]).areas(area);

    frame.render_widget(left, left_area);
    frame.render_widget(
        right
            .right_aligned()
            .style(Style::default().bg(THEME.panel)),
        right_area,
    );
}

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
}
