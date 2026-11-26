#[macro_use]
mod tester;

use {color_eyre::Result, glues_tui::input::KeyCode, tester::Tester, tui_textarea::CursorMove};

#[tokio::test]
async fn slash_search_and_vim_repeat_keys_move_between_matches() -> Result<()> {
    let mut t = Tester::new().await?;
    t.open_local().await?;
    t.open_first_note().await?;

    {
        let editor = t.app.context_mut().notebook.get_editor_mut();
        editor.select_all();
        editor.insert_str("alpha beta alpha\nalpha");
        editor.move_cursor(CursorMove::Jump(0, 0));
    }

    t.press('/').await;
    t.type_str("alpha").await;
    t.key(KeyCode::Enter).await;
    assert_eq!(t.app.context_mut().notebook.get_editor().cursor(), (0, 11));

    t.press('n').await;
    assert_eq!(t.app.context_mut().notebook.get_editor().cursor(), (1, 0));

    t.press('N').await;
    assert_eq!(t.app.context_mut().notebook.get_editor().cursor(), (0, 11));

    t.ctrl('p').await;
    assert!(t.app.context_mut().notebook.markdown_preview);
    assert!(t.app.context_mut().notebook.wrap_content);
    t.draw()?;
    assert!(t.snapshot_text().contains("[F2] Wrap on"));

    t.key(KeyCode::F(2)).await;
    assert!(!t.app.context_mut().notebook.wrap_content);
    t.draw()?;
    assert!(t.snapshot_text().contains("[F2] Wrap off"));

    t.key(KeyCode::F(2)).await;
    assert!(t.app.context_mut().notebook.wrap_content);

    Ok(())
}
