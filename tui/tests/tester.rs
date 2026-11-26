use {
    color_eyre::Result,
    glues_core::{CoreBackend, EntryEvent, backend::local::Db},
    glues_tui::{
        Action, App, config,
        input::{Input, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        logger,
    },
    ratatui::{Terminal, backend::TestBackend},
    std::sync::atomic::{AtomicU64, Ordering},
};

static TEST_NOTEBOOK_ID: AtomicU64 = AtomicU64::new(0);

pub struct Tester {
    pub app: App,
    pub term: Terminal<TestBackend>,
}

fn key_press(code: KeyCode, modifiers: KeyModifiers) -> Input {
    Input::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
    })
}

// Bring a concise snapshot macro into test files.
// Usage: snap!(t, "name");
#[allow(unused_macros)]
macro_rules! snap {
    ($t:expr, $name:expr) => {{
        let text = $t.snapshot_text();
        insta::assert_snapshot!($name, text);
    }};
}

impl Tester {
    pub async fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        std::fs::create_dir_all(cwd.join(".glues"))?;
        unsafe {
            std::env::set_var("HOME", &cwd);
        }
        config::init().await;
        logger::init().await;

        let backend = TestBackend::new(120, 40);
        let term = Terminal::new(backend)?;
        let app = App::new();
        Ok(Self { app, term })
    }

    pub fn draw(&mut self) -> color_eyre::Result<()> {
        self.term.draw(|f| self.app.draw(f))?;
        Ok(())
    }

    pub async fn press(&mut self, c: char) -> bool {
        self.handle_input(key_press(KeyCode::Char(c), KeyModifiers::NONE))
            .await
    }

    #[allow(dead_code)]
    pub async fn ctrl(&mut self, c: char) -> bool {
        self.handle_input(key_press(KeyCode::Char(c), KeyModifiers::CONTROL))
            .await
    }

    #[allow(dead_code)]
    pub async fn key(&mut self, code: KeyCode) -> bool {
        self.handle_input(key_press(code, KeyModifiers::NONE)).await
    }

    #[allow(dead_code)]
    pub async fn open_local(&mut self) -> Result<()> {
        let notebook_path = std::env::current_dir()?
            .join(".glues")
            .join("test-notebooks")
            .join(format!(
                "{}-{}",
                std::process::id(),
                TEST_NOTEBOOK_ID.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&notebook_path)?;

        let (task_tx, _task_rx) = std::sync::mpsc::channel();
        let path = notebook_path.to_string_lossy().into_owned();
        let mut db = Db::file(task_tx, &path).await?;
        let root_id = db.root_id();
        let note = db.add_note(root_id, "Sample Note".to_owned()).await?;
        db.update_note_content(note.id, "Hi :D".to_owned()).await?;
        drop(db);

        self.app
            .handle_action(
                Action::Dispatch(EntryEvent::OpenFile(path.clone()).into()),
                Input::Resize(120, 40),
            )
            .await;
        self.app.context_mut().notebook.storage_path = Some(notebook_path);
        self.draw()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn type_str(&mut self, s: &str) {
        for ch in s.chars() {
            let _ = self.press(ch).await;
        }
    }

    #[allow(dead_code)]
    pub async fn backspace(&mut self, n: usize) {
        for _ in 0..n {
            let _ = self.key(KeyCode::Backspace).await;
        }
    }

    #[allow(dead_code)]
    pub async fn open_first_note(&mut self) -> Result<()> {
        self.press('j').await;
        self.press('l').await;
        self.draw()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn snapshot_text(&self) -> String {
        buffer_lines(&self.term).join("\n")
    }

    async fn handle_input(&mut self, input: Input) -> bool {
        if !matches!(
            input,
            Input::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            })
        ) {
            return false;
        }

        match input {
            Input::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
                ..
            }) if modifiers.ctrl => true,
            _ => {
                let action = self.app.context_mut().consume(&input).await;
                self.app.handle_action(action, input).await
            }
        }
    }
}

fn buffer_lines(term: &Terminal<TestBackend>) -> Vec<String> {
    let buf = term.backend().buffer().clone();
    let area = buf.area();
    let mut lines = Vec::with_capacity(area.height as usize);
    for y in 0..area.height {
        let mut line = String::new();
        for x in 0..area.width {
            line.push_str(buf[(x, y)].symbol());
        }
        lines.push(line);
    }
    lines
}
