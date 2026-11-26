#[macro_use]
mod tester;
use tester::Tester;

use color_eyre::Result;
use glues_tui::input::KeyCode;

#[tokio::test]
async fn enter_visual_mode_and_escape() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;

    t.press('v').await;
    t.draw()?;
    snap!(t, "visual_mode");

    t.key(KeyCode::Esc).await;
    t.draw()?;
    snap!(t, "visual_back_to_normal");

    Ok(())
}

#[tokio::test]
async fn shift_v_yanks_complete_lines() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;
    {
        let notebook = &mut t.app.context_mut().notebook;
        let editor = notebook.get_editor_mut();
        editor.select_all();
        editor.insert_str("first\nsecond\nthird");
        editor.move_cursor(tui_textarea::CursorMove::Top);
        editor.move_cursor(tui_textarea::CursorMove::Down);
    }

    t.press('V').await;
    assert!(t.app.context_mut().notebook.visual_linewise);
    t.press('j').await;
    t.press('y').await;

    let notebook = &mut t.app.context_mut().notebook;
    assert_eq!(notebook.yank.as_deref(), Some("second\nthird"));
    assert!(notebook.line_yanked);
    assert_eq!(notebook.get_editor().lines(), ["first", "second", "third"]);
    Ok(())
}

#[tokio::test]
async fn shift_v_deletes_complete_lines() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;
    {
        let notebook = &mut t.app.context_mut().notebook;
        let editor = notebook.get_editor_mut();
        editor.select_all();
        editor.insert_str("first\nsecond\nthird");
        editor.move_cursor(tui_textarea::CursorMove::Top);
        editor.move_cursor(tui_textarea::CursorMove::Down);
    }

    t.press('V').await;
    t.press('j').await;
    t.press('d').await;

    let notebook = &mut t.app.context_mut().notebook;
    assert_eq!(notebook.get_editor().lines(), ["first"]);
    assert_eq!(notebook.yank.as_deref(), Some("second\nthird"));
    Ok(())
}

#[tokio::test]
async fn visual_numbering_move() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;

    t.press('v').await;
    t.press('2').await;
    t.draw()?;
    snap!(t, "visual_numbering");

    t.press('j').await;
    t.draw()?;
    snap!(t, "visual_move_2_down");

    Ok(())
}

#[tokio::test]
async fn visual_gateway_mode_top() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;

    t.press('v').await;
    t.press('g').await;
    t.draw()?;
    snap!(t, "visual_gateway");

    t.press('g').await;
    t.draw()?;
    snap!(t, "visual_move_top");

    Ok(())
}

#[tokio::test]
async fn editor_keymap_overlay_in_visual_mode() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;

    t.press('v').await;
    t.ctrl('h').await;
    t.draw()?;
    snap!(t, "visual_keymap_open");

    t.press('x').await;
    t.draw()?;
    snap!(t, "visual_keymap_closed");

    Ok(())
}
