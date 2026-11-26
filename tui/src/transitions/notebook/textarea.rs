use tui_textarea::{CursorMove, TextArea};

pub(super) fn cursor_move_forward(editor: &TextArea, n: usize) -> CursorMove {
    let row = editor.cursor().0;
    let col = editor.cursor().1;
    if col + n >= editor.lines()[row].len() {
        CursorMove::End
    } else {
        CursorMove::Jump(row as u16, (col + n) as u16)
    }
}

pub(super) fn cursor_move_back(editor: &TextArea, n: usize) -> CursorMove {
    let row = editor.cursor().0;
    let col = editor.cursor().1;
    if col < n {
        CursorMove::Head
    } else {
        CursorMove::Jump(row as u16, (col - n) as u16)
    }
}

pub(super) fn cursor_move_down(editor: &TextArea, n: usize) -> CursorMove {
    let num_lines = editor.lines().len();
    let row = editor.cursor().0;
    let col = editor.cursor().1;
    if row + n >= num_lines {
        CursorMove::Bottom
    } else {
        CursorMove::Jump((row + n) as u16, col as u16)
    }
}

pub(super) fn cursor_move_up(editor: &TextArea, n: usize) -> CursorMove {
    let row = editor.cursor().0;
    let col = editor.cursor().1;
    if row < n {
        CursorMove::Top
    } else {
        CursorMove::Jump((row - n) as u16, col as u16)
    }
}

pub(super) fn move_cursor_to_line_non_empty_start(editor: &mut TextArea) {
    editor.move_cursor(CursorMove::Head);

    let row = editor.cursor().0;
    let is_whitespace_at_first = editor.lines()[row]
        .chars()
        .next()
        .map(|c| c.is_whitespace())
        .unwrap_or(false);
    if is_whitespace_at_first {
        editor.move_cursor(CursorMove::WordForward);
    }
}

pub(super) fn move_cursor_word_end(editor: &mut TextArea, n: usize) {
    for _ in 0..n {
        editor.move_cursor(CursorMove::WordEnd);
    }
}

pub(super) fn move_cursor_word_back(editor: &mut TextArea, n: usize) {
    for _ in 0..n {
        editor.move_cursor(CursorMove::WordBack);
    }
}

pub(super) fn reselect_for_yank(editor: &mut TextArea) {
    let (begin, end) = match editor.selection_range() {
        None => return,
        Some(range) => range,
    };

    editor.cancel_selection();
    editor.move_cursor(CursorMove::Jump(begin.0 as u16, begin.1 as u16));
    editor.start_selection();
    editor.move_cursor(CursorMove::Jump(end.0 as u16, end.1 as u16));
    editor.move_cursor(CursorMove::Forward);
}

pub(super) fn select_linewise(editor: &mut TextArea) {
    let ((start_row, _), (end_row, _)) = editor.selection_range().unwrap_or_else(|| {
        let cursor = (editor.cursor().0, editor.cursor().1);
        (cursor, cursor)
    });
    editor.cancel_selection();
    editor.move_cursor(CursorMove::Jump(start_row as u16, 0));
    editor.start_selection();
    editor.move_cursor(CursorMove::Jump(end_row as u16, 0));
    editor.move_cursor(CursorMove::End);
}

pub(super) fn selected_lines(editor: &TextArea) -> (usize, usize, String) {
    let ((start_row, _), (end_row, _)) = editor.selection_range().unwrap_or_else(|| {
        let cursor = (editor.cursor().0, editor.cursor().1);
        (cursor, cursor)
    });
    let text = editor.lines()[start_row..=end_row].join("\n");
    (start_row, end_row, text)
}

pub(super) fn cut_selected_lines(editor: &mut TextArea) -> String {
    let (start_row, end_row, text) = selected_lines(editor);
    let line_count = editor.lines().len();
    editor.cancel_selection();
    editor.move_cursor(CursorMove::Jump(start_row as u16, 0));
    editor.start_selection();
    if end_row + 1 < line_count {
        editor.move_cursor(CursorMove::Jump((end_row + 1) as u16, 0));
    } else {
        editor.move_cursor(CursorMove::Jump(end_row as u16, 0));
        editor.move_cursor(CursorMove::End);
    }
    editor.cut();

    // Removing the final lines leaves their preceding newline at the end of
    // the buffer. Delete that separator too so the previous line remains.
    if end_row + 1 == line_count && start_row > 0 {
        editor.move_cursor(CursorMove::Jump(start_row as u16, 0));
        editor.delete_line_by_head();
    }
    editor.set_yank_text(text.clone());
    text
}

pub(super) fn switch_case(editor: &mut TextArea) {
    let yank = editor.yank_text();
    reselect_for_yank(editor);
    editor.cut();

    let changed = editor
        .yank_text()
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().to_string()
            } else {
                c.to_uppercase().to_string()
            }
        })
        .collect::<String>();

    editor.insert_str(changed);
    editor.set_yank_text(yank);
}
