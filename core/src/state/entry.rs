use crate::{
    EntryEvent, EntryTransition, Error, Event, Glues, Result,
    backend::local::Db,
    state::notebook::NotebookState,
    types::{KeymapGroup, KeymapItem},
};

pub struct EntryState;

impl EntryState {
    pub async fn consume(glues: &mut Glues, event: Event) -> Result<EntryTransition> {
        use EntryEvent::*;
        use Event::*;

        match event {
            Entry(OpenFile(path)) => {
                let db = Db::file(glues.task_tx.clone(), &path).await?;
                glues.db = Some(Box::new(db));
                glues.state = NotebookState::new(glues).await?.into();

                Ok(EntryTransition::OpenNotebook)
            }
            Entry(OpenGit {
                path,
                remote,
                branch,
            }) => {
                let db = Db::git(glues.task_tx.clone(), &path, remote, branch).await?;
                glues.db = Some(Box::new(db));
                glues.state = NotebookState::new(glues).await?.into();

                Ok(EntryTransition::OpenNotebook)
            }
            Key(_) => Ok(EntryTransition::Inedible(event)),
            Cancel => Ok(EntryTransition::None),
            _ => Err(Error::Todo("EntryState::consume".to_owned())),
        }
    }

    pub fn describe(&self) -> Result<String> {
        Ok(
            "Glues - TUI note-taking app offering complete data control and flexible storage options"
                .to_owned(),
        )
    }

    pub fn keymap(&self) -> Vec<KeymapGroup> {
        vec![
            KeymapGroup::new(
                "Navigation",
                vec![
                    KeymapItem::new("j", "Select next"),
                    KeymapItem::new("k", "Select previous"),
                ],
            ),
            KeymapGroup::new(
                "Actions",
                vec![
                    KeymapItem::new("Enter", "Run selected item"),
                    KeymapItem::new("q", "Quit"),
                ],
            ),
        ]
    }
}
