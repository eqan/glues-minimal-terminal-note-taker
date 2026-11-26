#[macro_use]
mod tester;
use tester::Tester;

use color_eyre::Result;

#[tokio::test]
async fn entry_menu_offers_local_and_git_storage() -> Result<()> {
    let mut t = Tester::new().await?;
    t.draw()?;
    let screen = t.snapshot_text();
    assert!(screen.contains("Local"), "{screen}");
    assert!(screen.contains("Git"), "{screen}");
    assert!(!screen.contains("Instant"), "{screen}");
    snap!(t, "entry_menu_local_git");

    Ok(())
}

#[tokio::test]
async fn quits_on_q() -> Result<()> {
    let mut t = Tester::new().await?;
    t.draw()?;

    let quit = t.press('q').await;
    assert!(quit);

    Ok(())
}

#[tokio::test]
async fn help_overlay_toggles() -> Result<()> {
    let mut t = Tester::new().await?;
    t.draw()?;

    // open help via shortcut
    t.press('h').await;
    t.draw()?;
    snap!(t, "help_open");

    // any key closes help
    t.press('x').await;
    t.draw()?;
    snap!(t, "help_closed");

    Ok(())
}

#[tokio::test]
async fn open_local_prompt() -> Result<()> {
    let mut t = Tester::new().await?;

    t.draw()?;
    // open Local storage prompt via hotkey 'l'
    t.press('l').await;
    t.draw()?;
    snap!(t, "local_prompt");

    Ok(())
}

#[tokio::test]
async fn theme_dialog_open() -> Result<()> {
    let mut t = Tester::new().await?;

    t.draw()?;
    t.press('t').await;
    t.draw()?;
    snap!(t, "theme_dialog_open");

    Ok(())
}
