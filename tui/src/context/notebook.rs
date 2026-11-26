mod tree_item;

use {
    crate::{
        action::{Action, TuiAction},
        input::{Input, KeyCode, KeyEvent, to_textarea_input},
        logger::*,
    },
    glues_core::{
        NotebookEvent,
        data::Note,
        state::notebook::{DirectoryItem, Tab},
        types::{Id, NoteId},
    },
    ratatui::{text::Line, widgets::ListState},
    std::{collections::HashMap, path::PathBuf},
    tui_textarea::TextArea,
};

use arboard::Clipboard;

pub use tree_item::{TreeItem, TreeItemKind};

pub const REMOVE_NOTE: &str = "Remove note";
pub const RENAME_NOTE: &str = "Rename note";

pub const ADD_NOTE: &str = "Add note";
pub const ADD_DIRECTORY: &str = "Add directory";
pub const RENAME_DIRECTORY: &str = "Rename directory";
pub const REMOVE_DIRECTORY: &str = "Remove directory";

pub const CLOSE: &str = "Close";

pub const NOTE_ACTIONS: [&str; 3] = [RENAME_NOTE, REMOVE_NOTE, CLOSE];
pub const DIRECTORY_ACTIONS: [&str; 5] = [
    ADD_NOTE,
    ADD_DIRECTORY,
    RENAME_DIRECTORY,
    REMOVE_DIRECTORY,
    CLOSE,
];

#[derive(Clone, Copy, PartialEq)]
pub enum ContextState {
    NoteTreeBrowsing,
    NoteTreeNumbering,
    NoteTreeGateway,
    NoteActionsDialog,
    DirectoryActionsDialog,
    MoveMode,
    EditorNormalMode { idle: bool },
    EditorVisualMode,
    EditorInsertMode,
}

impl ContextState {
    pub fn is_editor(&self) -> bool {
        matches!(
            self,
            ContextState::EditorNormalMode { .. }
                | ContextState::EditorInsertMode
                | ContextState::EditorVisualMode
        )
    }
}

pub struct NotebookContext {
    pub state: ContextState,

    // note tree
    pub tree_state: ListState,
    pub tree_items: Vec<TreeItem>,
    pub tree_width: u16,

    // note actions
    pub note_actions_state: ListState,

    // directory actions
    pub directory_actions_state: ListState,

    // editor
    pub editor_height: u16,
    pub markdown_preview: bool,
    pub markdown_preview_scroll: u16,
    pub wrap_content: bool,
    pub visual_linewise: bool,
    pub storage_path: Option<PathBuf>,
    pub last_search: Option<String>,
    pub last_search_backward: bool,
    pub tabs: Vec<Tab>,
    pub tab_index: Option<usize>,
    pub editors: HashMap<NoteId, EditorItem>,

    pub show_line_number: bool,
    pub show_browser: bool,
    pub line_yanked: bool,
    pub yank: Option<String>,
}

pub struct EditorItem {
    pub editor: TextArea<'static>,
    pub dirty: bool,
}

impl Default for NotebookContext {
    fn default() -> Self {
        Self {
            state: ContextState::NoteTreeBrowsing,
            tree_state: ListState::default().with_selected(Some(0)),
            tree_items: vec![],
            tree_width: 45,

            note_actions_state: ListState::default(),
            directory_actions_state: ListState::default(),

            editor_height: 0,
            markdown_preview: false,
            markdown_preview_scroll: 0,
            wrap_content: true,
            visual_linewise: false,
            storage_path: None,
            last_search: None,
            last_search_backward: false,
            tabs: vec![],
            tab_index: None,
            editors: HashMap::new(),

            show_line_number: true,
            show_browser: true,
            line_yanked: false,
            yank: None,
        }
    }
}

impl NotebookContext {
    pub fn search(&mut self, pattern: &str, backward: bool) -> Result<bool, String> {
        if pattern.is_empty() {
            return Ok(true);
        }

        let found = search_text_area(self.get_editor_mut(), pattern, backward)?;
        self.last_search = Some(pattern.to_owned());
        self.last_search_backward = backward;
        Ok(found)
    }

    pub fn repeat_search(&mut self, reverse: bool) -> Option<Result<bool, String>> {
        let pattern = self.last_search.clone()?;
        let backward = self.last_search_backward ^ reverse;
        Some(search_text_area(self.get_editor_mut(), &pattern, backward))
    }

    pub fn get_opened_note(&self) -> Option<&Note> {
        self.tab_index
            .and_then(|i| self.tabs.get(i))
            .map(|t| &t.note)
    }

    pub fn get_editor(&self) -> &TextArea<'static> {
        let note_id = &self
            .tab_index
            .and_then(|i| self.tabs.get(i))
            .log_expect("[NotebookContext::get_editor] no opened note")
            .note
            .id;

        &self
            .editors
            .get(note_id)
            .log_expect("[NotebookContext::get_editor] editor not found")
            .editor
    }

    pub fn get_editor_mut(&mut self) -> &mut TextArea<'static> {
        let note_id = &self
            .tab_index
            .and_then(|i| self.tabs.get(i))
            .log_expect("[NotebookContext::get_editor_mut] no opened note")
            .note
            .id;

        &mut self
            .editors
            .get_mut(note_id)
            .log_expect("[NotebookContext::get_editor_mut] editor not found")
            .editor
    }

    pub fn mark_dirty(&mut self) {
        if let Some(editor_item) = self
            .tab_index
            .and_then(|i| self.tabs.get(i))
            .and_then(|tab| self.editors.get_mut(&tab.note.id))
        {
            editor_item.dirty = true;
        }
    }

    pub fn mark_clean(&mut self, note_id: &NoteId) {
        if let Some(editor_item) = self.editors.get_mut(note_id) {
            editor_item.dirty = false;
        }
    }

    pub fn update_items(&mut self, directory_item: &DirectoryItem) {
        self.tree_items = self.flatten(directory_item, 0, true);
    }

    fn flatten(
        &self,
        directory_item: &DirectoryItem,
        depth: usize,
        selectable: bool,
    ) -> Vec<TreeItem> {
        let id = self
            .tree_state
            .selected()
            .and_then(|i| self.tree_items.get(i))
            .map(|item| item.id());
        let is_move_mode = matches!(self.state, ContextState::MoveMode);
        let selectable = !is_move_mode || (selectable && Some(&directory_item.directory.id) != id);

        let mut items = vec![TreeItem {
            depth,
            target: Some(&directory_item.directory.id) == id,
            selectable,
            kind: TreeItemKind::Directory {
                directory: directory_item.directory.clone(),
                opened: directory_item.children.is_some(),
            },
        }];

        if let Some(children) = &directory_item.children {
            for item in &children.directories {
                items.extend(self.flatten(item, depth + 1, selectable));
            }

            for note in &children.notes {
                items.push(TreeItem {
                    depth: depth + 1,
                    target: Some(&note.id) == id,
                    selectable: !is_move_mode,
                    kind: TreeItemKind::Note { note: note.clone() },
                })
            }
        }

        items
    }

    pub fn select_item(&mut self, id: &Id) {
        for (i, item) in self.tree_items.iter().enumerate() {
            if item.id() == id {
                self.tree_state.select(Some(i));
                break;
            }
        }
    }

    pub fn select_first(&mut self) {
        let i = self
            .tree_items
            .iter()
            .enumerate()
            .find(|(_, item)| item.selectable)
            .map(|(i, _)| i);

        if i.is_some() {
            self.tree_state.select(i);
        }
    }

    pub fn select_last(&mut self) {
        let i = self
            .tree_items
            .iter()
            .enumerate()
            .rev()
            .find(|(_, item)| item.selectable)
            .map(|(i, _)| i);

        if i.is_some() {
            self.tree_state.select(i);
        }
    }

    pub fn select_next(&mut self, step: usize) {
        let i = match self.tree_state.selected().unwrap_or_default() + step {
            i if i >= self.tree_items.len() => self.tree_items.len() - 1,
            i => i,
        };

        let i = self
            .tree_items
            .iter()
            .enumerate()
            .skip(i)
            .find(|(_, item)| item.selectable)
            .map(|(i, _)| i);

        if i.is_some() {
            self.tree_state.select(i);
        }
    }

    pub fn select_prev(&mut self, step: usize) {
        let i = self
            .tree_state
            .selected()
            .unwrap_or_default()
            .saturating_sub(step);

        let i = self
            .tree_items
            .iter()
            .enumerate()
            .rev()
            .skip(self.tree_items.len() - i - 1)
            .find(|(_, item)| item.selectable)
            .map(|(i, _)| i);

        if i.is_some() {
            self.tree_state.select(i);
        }
    }

    pub fn select_next_dir(&mut self) {
        let i = self.tree_state.selected().unwrap_or_default() + 1;

        if i >= self.tree_items.len() {
            return;
        }

        let i = self
            .tree_items
            .iter()
            .enumerate()
            .skip(i)
            .filter(|(_, item)| item.is_directory())
            .find(|(_, item)| item.selectable)
            .map(|(i, _)| i);

        if i.is_some() {
            self.tree_state.select(i);
        }
    }

    pub fn select_prev_dir(&mut self) {
        let i = self
            .tree_state
            .selected()
            .unwrap_or_default()
            .saturating_sub(1);

        let i = self
            .tree_items
            .iter()
            .enumerate()
            .rev()
            .skip(self.tree_items.len() - i - 1)
            .filter(|(_, item)| item.is_directory())
            .find(|(_, item)| item.selectable)
            .map(|(i, _)| i);

        if i.is_some() {
            self.tree_state.select(i);
        }
    }

    pub fn selected(&self) -> &TreeItem {
        self.tree_state
            .selected()
            .and_then(|i| self.tree_items.get(i))
            .log_expect("[NotebookContext::selected] selected must not be empty")
    }

    pub fn open_note(&mut self, note_id: NoteId, content: String) {
        let item = EditorItem {
            editor: TextArea::from(content.lines()),
            dirty: false,
        };

        self.editors.insert(note_id, item);
    }

    pub fn apply_yank(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        if let Some(yank) = self.yank.as_ref().cloned() {
            self.get_editor_mut().set_yank_text(yank);
        }
    }

    pub fn update_yank(&mut self) {
        let text = self.get_editor().yank_text();

        if let Ok(mut clipboard) = Clipboard::new() {
            let _ = clipboard.set_text(&text);
        }

        self.yank = Some(text);
    }

    pub fn paste_from_clipboard(&mut self) -> Result<(), String> {
        let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
        if let Ok(image) = clipboard.get_image() {
            self.insert_clipboard_image(image)?;
            return Ok(());
        }

        let text = clipboard
            .get_text()
            .map_err(|_| "Clipboard does not contain text or a supported image".to_owned())?;
        self.get_editor_mut().insert_str(text);
        self.mark_dirty();
        Ok(())
    }

    pub fn insert_clipboard_image(&mut self, image: arboard::ImageData) -> Result<String, String> {
        let notebook_path = self
            .storage_path
            .clone()
            .ok_or_else(|| "Open a Local or Git notebook before pasting an image".to_owned())?;
        let markdown = save_clipboard_image(&notebook_path, image)?;
        self.get_editor_mut().insert_str(&markdown);
        self.mark_dirty();
        Ok(markdown)
    }

    pub fn consume(&mut self, input: &Input) -> Action {
        if let Input::Key(key) = input
            && key.modifiers.ctrl
            && matches!(
                key.code,
                KeyCode::Char('\\') | KeyCode::Char('o') | KeyCode::Char('O')
            )
        {
            if matches!(self.state, ContextState::NoteTreeBrowsing)
                && self.show_browser
                && self.tabs.is_empty()
            {
                return Action::None;
            }
            return TuiAction::ToggleSidePane.into();
        }

        if let Input::Key(key) = input
            && key.code == KeyCode::F(2)
            && !key.modifiers.ctrl
            && !key.modifiers.alt
            && (self.markdown_preview || self.state.is_editor())
        {
            self.wrap_content = !self.wrap_content;
            return Action::None;
        }

        if let Input::Key(key) = input
            && !key.modifiers.ctrl
            && !key.modifiers.alt
            && !self.markdown_preview
        {
            let is_linewise = matches!(key.code, KeyCode::Char('V'))
                || (key.code == KeyCode::Char('v') && key.modifiers.shift);
            let is_charwise = key.code == KeyCode::Char('v') && !key.modifiers.shift;

            match self.state {
                ContextState::EditorNormalMode { idle: true } if is_linewise => {
                    self.visual_linewise = true;
                }
                ContextState::EditorNormalMode { idle: true } if is_charwise => {
                    self.visual_linewise = false;
                }
                ContextState::EditorVisualMode if is_linewise || is_charwise => {
                    self.visual_linewise = is_linewise;
                    if is_linewise {
                        select_linewise(self.get_editor_mut());
                    }
                    return Action::None;
                }
                _ => {}
            }
        }

        if self.markdown_preview {
            return self.consume_on_markdown_preview(input);
        }

        if matches!(self.state, ContextState::EditorInsertMode)
            && let Input::Paste(text) = input
        {
            self.get_editor_mut().insert_str(text);
            self.mark_dirty();
            return Action::None;
        }

        let code = match input {
            Input::Key(key) => key.code,
            _ => return Action::None,
        };

        match self.state {
            ContextState::NoteTreeBrowsing => self.consume_on_note_tree_browsing(code),
            ContextState::NoteTreeGateway
            | ContextState::NoteTreeNumbering
            | ContextState::MoveMode => Action::PassThrough,
            ContextState::EditorNormalMode { idle } => self.consume_on_editor_normal(input, idle),
            ContextState::EditorVisualMode => Action::PassThrough,
            ContextState::EditorInsertMode => self.consume_on_editor_insert(input),
            ContextState::NoteActionsDialog => self.consume_on_note_actions(code),
            ContextState::DirectoryActionsDialog => self.consume_on_directory_actions(code),
        }
    }

    fn consume_on_markdown_preview(&mut self, input: &Input) -> Action {
        let key = match input {
            Input::Key(key) => key,
            _ => return Action::None,
        };

        if key.code == KeyCode::Char('p') && key.modifiers.ctrl {
            self.markdown_preview = false;
            self.markdown_preview_scroll = 0;
            return Action::None;
        }

        match key.code {
            KeyCode::Esc => {
                self.markdown_preview = false;
                self.markdown_preview_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.markdown_preview_scroll = self.markdown_preview_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.markdown_preview_scroll = self.markdown_preview_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.markdown_preview_scroll = self
                    .markdown_preview_scroll
                    .saturating_add(self.editor_height.max(1));
            }
            KeyCode::PageUp => {
                self.markdown_preview_scroll = self
                    .markdown_preview_scroll
                    .saturating_sub(self.editor_height.max(1));
            }
            KeyCode::Home => self.markdown_preview_scroll = 0,
            KeyCode::Tab => {
                self.markdown_preview = false;
                self.markdown_preview_scroll = 0;
                self.show_browser = true;
                self.update_yank();
                return TuiAction::SaveAndPassThrough.into();
            }
            _ => {}
        }

        Action::None
    }

    fn consume_on_note_tree_browsing(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Char('m') => {
                if self
                    .tree_state
                    .selected()
                    .and_then(|idx| self.tree_items.get(idx))
                    .log_expect("[NotebookContext::consume] selected must not be empty")
                    .is_directory()
                {
                    self.directory_actions_state.select_first();
                } else {
                    self.note_actions_state.select_first();
                }

                Action::PassThrough
            }
            KeyCode::Esc => TuiAction::OpenNotebookQuitMenu {
                save_before_open: false,
            }
            .into(),
            _ => Action::PassThrough,
        }
    }

    fn consume_on_editor_normal(&mut self, input: &Input, idle: bool) -> Action {
        let code = match input {
            Input::Key(key) => key.code,
            _ => return Action::None,
        };

        if idle
            && let Input::Key(key) = input
            && key.code == KeyCode::Char('p')
            && key.modifiers.ctrl
        {
            self.markdown_preview = true;
            self.markdown_preview_scroll = 0;
            return Action::None;
        }

        match code {
            KeyCode::Char('/') if idle => TuiAction::Prompt {
                message: vec![Line::raw("Search forward:")],
                action: Box::new(TuiAction::Search { backward: false }.into()),
                default: self.last_search.clone(),
            }
            .into(),
            KeyCode::Char('n') if idle && self.last_search.is_some() => {
                TuiAction::RepeatSearch { reverse: false }.into()
            }
            KeyCode::Char('N') if idle && self.last_search.is_some() => {
                TuiAction::RepeatSearch { reverse: true }.into()
            }
            KeyCode::Esc if idle => TuiAction::OpenNotebookQuitMenu {
                save_before_open: true,
            }
            .into(),
            KeyCode::Tab if idle => {
                self.show_browser = true;
                self.update_yank();

                TuiAction::SaveAndPassThrough.into()
            }
            _ => Action::PassThrough,
        }
    }

    fn consume_on_editor_insert(&mut self, input: &Input) -> Action {
        match input {
            Input::Key(KeyEvent {
                code: KeyCode::Char('v' | 'V'),
                modifiers,
                ..
            }) if modifiers.ctrl => match self.paste_from_clipboard() {
                Ok(()) => Action::None,
                Err(error) => TuiAction::Alert(error).into(),
            },
            Input::Key(KeyEvent {
                code: KeyCode::Char('p'),
                modifiers,
                ..
            }) if modifiers.ctrl => {
                self.markdown_preview = true;
                self.markdown_preview_scroll = 0;
                Action::Dispatch(NotebookEvent::ViewNote.into())
            }
            Input::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => Action::Dispatch(NotebookEvent::ViewNote.into()),
            Input::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers,
                ..
            }) if modifiers.ctrl => TuiAction::ShowEditorKeymap.into(),
            Input::Key(KeyEvent {
                code: KeyCode::Char('c' | 'x' | 'w' | 'k' | 'j'),
                modifiers,
                ..
            }) if modifiers.ctrl => {
                self.line_yanked = false;
                if let Some(text_input) = to_textarea_input(input) {
                    self.get_editor_mut().input(text_input);
                }
                Action::None
            }
            _ => {
                if let Some(text_input) = to_textarea_input(input) {
                    self.get_editor_mut().input(text_input);
                }
                Action::None
            }
        }
    }

    fn consume_on_note_actions(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.note_actions_state.select_next();
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.note_actions_state.select_previous();
                Action::None
            }
            KeyCode::Esc => Action::Dispatch(NotebookEvent::CloseNoteActionsDialog.into()),
            KeyCode::Enter => {
                match NOTE_ACTIONS[self
                    .note_actions_state
                    .selected()
                    .log_expect("note action must not be empty")]
                {
                    RENAME_NOTE => TuiAction::Prompt {
                        message: vec![Line::raw("Enter new note name:")],
                        action: Box::new(TuiAction::RenameNote.into()),
                        default: Some(self.selected().name()),
                    }
                    .into(),
                    REMOVE_NOTE => TuiAction::Confirm {
                        message: "Confirm to remove note?".to_owned(),
                        action: Box::new(TuiAction::RemoveNote.into()),
                    }
                    .into(),
                    CLOSE => Action::Dispatch(NotebookEvent::CloseNoteActionsDialog.into()),
                    _ => Action::None,
                }
            }
            _ => Action::PassThrough,
        }
    }

    fn consume_on_directory_actions(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.directory_actions_state.select_next();
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.directory_actions_state.select_previous();
                Action::None
            }
            KeyCode::Enter => {
                match DIRECTORY_ACTIONS[self
                    .directory_actions_state
                    .selected()
                    .log_expect("directory action must not be empty")]
                {
                    ADD_NOTE => TuiAction::Prompt {
                        message: vec![Line::raw("Enter note name:")],
                        action: Box::new(TuiAction::AddNote.into()),
                        default: None,
                    }
                    .into(),
                    ADD_DIRECTORY => TuiAction::Prompt {
                        message: vec![Line::raw("Enter directory name:")],
                        action: Box::new(TuiAction::AddDirectory.into()),
                        default: None,
                    }
                    .into(),
                    RENAME_DIRECTORY => TuiAction::Prompt {
                        message: vec![Line::raw("Enter new directory name:")],
                        action: Box::new(TuiAction::RenameDirectory.into()),
                        default: Some(self.selected().name()),
                    }
                    .into(),
                    REMOVE_DIRECTORY => TuiAction::Confirm {
                        message: "Confirm to remove directory?".to_owned(),
                        action: Box::new(TuiAction::RemoveDirectory.into()),
                    }
                    .into(),
                    CLOSE => Action::Dispatch(NotebookEvent::CloseDirectoryActionsDialog.into()),
                    _ => Action::None,
                }
            }
            KeyCode::Esc => Action::Dispatch(NotebookEvent::CloseDirectoryActionsDialog.into()),
            _ => Action::PassThrough,
        }
    }
}

fn select_linewise(editor: &mut TextArea<'_>) {
    let ((start_row, _), (end_row, _)) = editor.selection_range().unwrap_or_else(|| {
        let cursor = (editor.cursor().0, editor.cursor().1);
        (cursor, cursor)
    });
    editor.cancel_selection();
    editor.move_cursor(tui_textarea::CursorMove::Jump(start_row as u16, 0));
    editor.start_selection();
    editor.move_cursor(tui_textarea::CursorMove::Jump(end_row as u16, 0));
    editor.move_cursor(tui_textarea::CursorMove::End);
}

fn save_clipboard_image(
    notebook_path: &std::path::Path,
    image: arboard::ImageData,
) -> Result<String, String> {
    let width = u32::try_from(image.width).map_err(|error| error.to_string())?;
    let height = u32::try_from(image.height).map_err(|error| error.to_string())?;
    if width == 0 || height == 0 {
        return Err("Clipboard image is empty".to_owned());
    }

    let asset_dir = notebook_path.join("assets");
    std::fs::create_dir_all(&asset_dir).map_err(|error| error.to_string())?;
    let filename = format!("{}.png", uuid::Uuid::now_v7());
    let image_path = asset_dir.join(&filename);
    let pixels = image::RgbaImage::from_raw(width, height, image.bytes.into_owned())
        .ok_or_else(|| "Clipboard image data has an invalid pixel size".to_owned())?;
    image::DynamicImage::ImageRgba8(pixels)
        .save_with_format(&image_path, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;

    Ok(format!("![Clipboard image](assets/{filename})"))
}

fn search_text_area(
    editor: &mut TextArea<'_>,
    pattern: &str,
    backward: bool,
) -> Result<bool, String> {
    editor
        .set_search_pattern(pattern)
        .map_err(|error| error.to_string())?;
    Ok(if backward {
        editor.search_back(false)
    } else {
        editor.search_forward(false)
    })
}

#[cfg(test)]
mod search_tests {
    use super::search_text_area;
    use super::{ContextState, NotebookContext};
    use crate::{
        action::Action,
        input::{Input, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    };
    use tui_textarea::{CursorMove, TextArea};

    #[test]
    fn forward_regex_search_finds_next_match_and_wraps() {
        let mut editor = TextArea::from(["one two one", "one"]);
        assert!(search_text_area(&mut editor, "o+ne", false).unwrap());
        assert_eq!(editor.cursor(), (0, 8));
        assert!(search_text_area(&mut editor, "o+ne", false).unwrap());
        assert_eq!(editor.cursor(), (1, 0));
        assert!(search_text_area(&mut editor, "o+ne", false).unwrap());
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn backward_search_finds_previous_match_and_wraps() {
        let mut editor = TextArea::from(["one two one", "one"]);
        editor.move_cursor(CursorMove::Jump(1, 0));
        assert!(search_text_area(&mut editor, "one", true).unwrap());
        assert_eq!(editor.cursor(), (0, 8));
        assert!(search_text_area(&mut editor, "one", true).unwrap());
        assert_eq!(editor.cursor(), (0, 0));
        assert!(search_text_area(&mut editor, "one", true).unwrap());
        assert_eq!(editor.cursor(), (1, 0));
    }

    #[test]
    fn search_supports_regular_expressions_and_rejects_invalid_patterns() {
        let mut editor = TextArea::from(["one two", "three"]);
        assert!(search_text_area(&mut editor, "(one|three)", false).unwrap());
        assert_eq!(editor.cursor(), (1, 0));
        assert!(search_text_area(&mut editor, "(", false).is_err());
    }

    #[test]
    fn missing_search_returns_none() {
        let mut editor = TextArea::from(["notes"]);
        assert!(!search_text_area(&mut editor, "absent", false).unwrap());
    }

    #[test]
    fn f2_toggles_content_wrapping_in_the_editor_and_preview() {
        let mut context = NotebookContext {
            state: ContextState::EditorNormalMode { idle: true },
            ..NotebookContext::default()
        };
        let f2 = Input::Key(KeyEvent {
            code: KeyCode::F(2),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
        });

        assert!(matches!(context.consume(&f2), Action::None));
        assert!(!context.wrap_content);

        context.markdown_preview = true;
        assert!(matches!(context.consume(&f2), Action::None));
        assert!(context.wrap_content);
    }
}
