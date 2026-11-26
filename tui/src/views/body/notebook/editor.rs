use {
    crate::{
        context::{Context, notebook::ContextState},
        theme::THEME,
        views::statusbar,
    },
    ratatui::{
        Frame,
        layout::Rect,
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::{Block, Padding, Paragraph, Wrap},
    },
    tui_textarea::{TextArea, WrapMode},
};

const NOTE_SYMBOL: &str = "󱇗 ";
const SAMPLE_NOTE: [&str; 7] = [
    "Welcome to Glues!",
    "",
    "Press `?` to see keymaps and shortcuts.",
    "Press `m` in the note tree to create notes or directories.",
    "Press `Enter` on a note to open it and start writing.",
    "",
    "GitHub: https://github.com/eqan/glues-minimal-terminal-note-taker",
];

pub fn draw(frame: &mut Frame, area: Rect, context: &mut Context) {
    context.notebook.editor_height = area.height - 2;

    let title = if let Some(tab_index) = context.notebook.tab_index {
        let mut title = vec![];
        for (i, tab) in context.notebook.tabs.iter().enumerate() {
            let name = format!(" {NOTE_SYMBOL}{} ", tab.note.name);
            let name = if i == tab_index {
                if context.notebook.state.is_editor() {
                    name.fg(THEME.accent_text).bg(THEME.accent)
                } else {
                    name.fg(THEME.accent_text).bg(THEME.inactive_bg)
                }
            } else {
                name.fg(THEME.hint)
            };

            title.push(name);
        }

        Line::from(title)
    } else {
        Line::from("[Editor]".fg(THEME.inactive_text))
    };

    let (mode, bg) = if context.notebook.markdown_preview {
        (
            Span::raw(" PREVIEW ")
                .fg(THEME.accent_text)
                .bg(THEME.accent),
            THEME.accent,
        )
    } else {
        match context.notebook.state {
            ContextState::EditorNormalMode { .. } => (
                Span::raw(" NORMAL ").fg(THEME.text).bg(THEME.background),
                THEME.background,
            ),
            ContextState::EditorInsertMode => (
                Span::raw(" INSERT ").fg(THEME.accent_text).bg(THEME.accent),
                THEME.accent,
            ),
            ContextState::EditorVisualMode => (
                Span::raw(" VISUAL ").fg(THEME.error_text).bg(THEME.error),
                THEME.error,
            ),
            _ => (Span::raw("        ").bg(THEME.surface), THEME.surface),
        }
    };

    let mut bottom_left = vec![mode];
    if context.notebook.tab_index.is_some() {
        bottom_left.push(Span::raw("").fg(bg).bg(THEME.background));
        let wrapping = if context.notebook.wrap_content {
            "on"
        } else {
            "off"
        };
        bottom_left.push(
            Span::raw(format!(" [F2] Wrap {wrapping} "))
                .fg(THEME.hint)
                .bg(THEME.background),
        );
    }

    let bottom_left = Line::from(bottom_left);
    let block = Block::new().title(title).title_bottom(bottom_left);
    let mut bottom_right = match (
        context.last_log.as_ref(),
        context.notebook.editors.iter().any(|(_, item)| item.dirty),
    ) {
        (_, true) => vec![
            Span::raw("").fg(THEME.accent).bg(THEME.background),
            Span::raw(" 󰔚 Saving... ")
                .fg(THEME.accent_text)
                .bg(THEME.accent),
        ],
        (Some((log, _)), false) => vec![
            Span::raw(format!(" {log} "))
                .fg(THEME.success_text)
                .bg(THEME.success),
        ],
        (None, false) => vec![],
    };
    if !bottom_right.is_empty() {
        bottom_right.push(Span::raw(" "));
    }
    bottom_right.extend(statusbar::controls(&context.notebook).spans);
    let block = block
        .title_bottom(Line::from(bottom_right).right_aligned())
        .fg(THEME.text)
        .bg(THEME.background)
        .padding(if context.notebook.show_line_number {
            Padding::ZERO
        } else {
            Padding::left(1)
        });

    if context.notebook.markdown_preview {
        let content = context
            .notebook
            .tab_index
            .map(|_| context.notebook.get_editor().lines().join("\n"))
            .unwrap_or_else(|| SAMPLE_NOTE.join("\n"));
        let markdown = super::markdown::render_with_images(
            &content,
            context.notebook.storage_path.as_deref(),
            area.width.saturating_sub(6).clamp(1, 80),
        );
        let preview_block = block.padding(Padding::left(1));

        let paragraph = Paragraph::new(markdown)
            .block(preview_block)
            .style(Style::default().fg(THEME.text).bg(THEME.background));
        let paragraph = if context.notebook.wrap_content {
            paragraph.wrap(Wrap { trim: false })
        } else {
            paragraph
        }
        .scroll((context.notebook.markdown_preview_scroll, 0));

        frame.render_widget(paragraph, area);

        return;
    }

    let show_line_number = context.notebook.show_line_number;
    let wrap_content = context.notebook.wrap_content;
    let state = context.notebook.state;
    let mut editor = TextArea::from(SAMPLE_NOTE);
    let editor = if context.notebook.tab_index.is_some() {
        context.notebook.get_editor_mut()
    } else {
        &mut editor
    };

    editor.set_wrap_mode(if wrap_content {
        WrapMode::Word
    } else {
        WrapMode::None
    });

    editor.set_block(block);

    let (cursor_style, cursor_line_style) = match state {
        ContextState::EditorNormalMode { .. }
        | ContextState::EditorInsertMode
        | ContextState::EditorVisualMode => (
            Style::default().fg(THEME.accent_text).bg(THEME.accent),
            Style::default().underlined(),
        ),
        _ => (Style::default(), Style::default()),
    };

    editor.set_cursor_style(cursor_style);
    editor.set_cursor_line_style(cursor_line_style);
    editor.set_selection_style(Style::default().fg(THEME.accent_text).bg(THEME.accent));
    if show_line_number {
        editor.set_line_number_style(Style::default().fg(THEME.inactive_text));
    } else {
        editor.remove_line_number();
    }

    frame.render_widget(&*editor, area);
}
