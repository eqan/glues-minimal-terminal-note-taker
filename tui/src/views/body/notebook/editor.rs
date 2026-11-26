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
        text::Line,
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
    if area.height == 0 {
        return;
    }

    let content_area = Rect {
        height: area.height.saturating_sub(1),
        ..area
    };
    let footer_area = Rect {
        y: area.y + content_area.height,
        height: 1,
        ..area
    };
    context.notebook.editor_height = content_area.height.saturating_sub(2);

    draw_content(frame, content_area, context);

    let dirty = context.notebook.editors.values().any(|item| item.dirty);
    statusbar::draw(
        frame,
        footer_area,
        &context.notebook,
        context
            .last_log
            .as_ref()
            .map(|(message, _)| message.as_str()),
        dirty,
    );
}

fn draw_content(frame: &mut Frame, area: Rect, context: &mut Context) {
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

    let block = Block::new()
        .title(title)
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
