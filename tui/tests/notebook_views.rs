#[macro_use]
mod tester;
use tester::Tester;

use {
    color_eyre::Result,
    glues_core::transition::VimKeymapKind,
    glues_tui::{context::notebook::ContextState, input::KeyCode},
    std::time::SystemTime,
};

#[tokio::test]
async fn notebook_browser_toggle() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_instant().await?;
    t.open_first_note().await?;

    t.draw()?;
    snap!(t, "notebook_browser_visible");

    t.press('t').await;
    t.press('b').await;
    t.draw()?;
    snap!(t, "notebook_browser_hidden");

    t.press('t').await;
    t.press('b').await;

    Ok(())
}

#[tokio::test]
async fn markdown_preview_renders_note_content() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_instant().await?;
    t.open_first_note().await?;

    {
        let context = t.app.context_mut();
        let item = context
            .notebook
            .editors
            .values_mut()
            .next()
            .expect("opened note editor");
        item.editor.select_all();
        item.editor.insert_str(
            "# Heading\n\nA **bold _nested_ word** and [link](https://example.com).\n\n| Programme | Ranking | Assessment |\n|---|---:|---|\n| Alpha Robotics | #73 | Strong research and practical work |\n\n```rs\nlet x = 1;\n```",
        );
    }

    t.press('i').await;
    t.ctrl('p').await;
    assert!(t.app.context_mut().notebook.markdown_preview);

    t.draw()?;
    let rendered = t.snapshot_text();
    assert!(rendered.contains("Heading"), "{rendered}");
    assert!(rendered.contains("bold nested word"), "{rendered}");
    assert!(rendered.contains("◆ Alpha Robotics"), "{rendered}");
    assert!(rendered.contains("Ranking: #73"), "{rendered}");
    assert!(rendered.contains("Strong research and practical work"));
    assert!(rendered.contains("let x = 1;"), "{rendered}");
    assert!(!rendered.contains("**bold"));
    assert!(!rendered.contains("```"));
    assert!(!rendered.contains("|---"));
    Ok(())
}

#[tokio::test]
async fn notebook_editor_states() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_instant().await?;
    t.open_first_note().await?;

    t.press('i').await;
    t.type_str("X").await;
    t.key(KeyCode::Esc).await;
    t.draw()?;
    snap!(t, "editor_dirty");

    {
        let context = t.app.context_mut();
        if let Some((_, item)) = context.notebook.editors.iter_mut().next() {
            item.dirty = false;
        }
        context.last_log = Some(("Synced".into(), SystemTime::now()));
    }
    t.draw()?;
    snap!(t, "editor_last_log");
    t.app.context_mut().last_log = None;

    t.press('i').await;
    t.draw()?;
    snap!(t, "editor_insert_mode");
    t.key(KeyCode::Esc).await;

    t.press('v').await;
    t.draw()?;
    snap!(t, "editor_visual_mode");
    t.key(KeyCode::Esc).await;

    t.press('t').await;
    t.press('n').await;
    t.draw()?;
    snap!(t, "editor_no_line_numbers");
    t.press('t').await;
    t.press('n').await;

    // Add a second note to create multiple tabs
    t.key(KeyCode::Tab).await;
    t.press('m').await;
    t.key(KeyCode::Enter).await;
    t.type_str("Second").await;
    t.key(KeyCode::Enter).await;
    t.press('j').await;
    t.press('l').await;

    // Highlight tabs while browsing the tree
    t.key(KeyCode::Tab).await;
    {
        let context = t.app.context_mut();
        if context.notebook.tabs.len() == 1 {
            let mut tab = context.notebook.tabs[0].clone();
            tab.note.name = "Other".into();
            context.notebook.tabs.push(tab);
        }
    }
    t.draw()?;
    snap!(t, "editor_tab_tree");
    t.press('l').await;

    // Create a nested directory and note to exercise breadcrumb styling
    t.key(KeyCode::Tab).await;
    t.press('k').await;
    t.press('k').await;
    t.press('m').await;
    t.press('j').await;
    t.key(KeyCode::Enter).await;
    t.type_str("Dir").await;
    t.key(KeyCode::Enter).await;
    t.press('j').await;
    t.press('l').await;
    t.press('m').await;
    t.key(KeyCode::Enter).await;
    t.type_str("Nested").await;
    t.key(KeyCode::Enter).await;
    t.press('j').await;
    t.press('l').await;
    t.key(KeyCode::Esc).await;
    {
        let context = t.app.context_mut();
        for item in context.notebook.editors.values_mut() {
            item.dirty = false;
        }
        if let Some(tab) = context.notebook.tabs.get_mut(0) {
            tab.breadcrumb = vec!["Notes".into(), "Dir".into(), "Nested".into()];
        }
    }
    t.draw()?;
    snap!(t, "editor_breadcrumb_nested");

    t.press('t').await;
    t.press('x').await;
    t.draw()?;
    snap!(t, "editor_inactive");

    Ok(())
}

#[tokio::test]
async fn vim_keymap_variants() -> Result<()> {
    use ContextState::*;

    let mut t = Tester::new().await?;
    t.open_instant().await?;
    t.open_first_note().await?;

    let cases = [
        (
            EditorNormalMode { idle: true },
            VimKeymapKind::NormalIdle,
            "vim_keymap_normal_idle",
        ),
        (
            EditorNormalMode { idle: false },
            VimKeymapKind::NormalNumbering,
            "vim_keymap_normal_numbering",
        ),
        (
            EditorNormalMode { idle: false },
            VimKeymapKind::NormalDelete,
            "vim_keymap_normal_delete",
        ),
        (
            EditorNormalMode { idle: false },
            VimKeymapKind::NormalDelete2,
            "vim_keymap_normal_delete2",
        ),
        (
            EditorNormalMode { idle: false },
            VimKeymapKind::NormalChange,
            "vim_keymap_normal_change",
        ),
        (
            EditorNormalMode { idle: false },
            VimKeymapKind::NormalChange2,
            "vim_keymap_normal_change2",
        ),
        (
            EditorVisualMode,
            VimKeymapKind::VisualIdle,
            "vim_keymap_visual_idle",
        ),
        (
            EditorVisualMode,
            VimKeymapKind::VisualNumbering,
            "vim_keymap_visual_numbering",
        ),
    ];

    for (state, kind, name) in cases {
        {
            let context = t.app.context_mut();
            context.notebook.state = state;
            context.vim_keymap = Some(kind);
        }
        t.draw()?;
        snap!(t, name);
        t.app.context_mut().vim_keymap = None;
    }

    t.key(KeyCode::Esc).await;

    Ok(())
}
