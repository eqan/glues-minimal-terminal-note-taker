use {
    crate::{Error, Result, schema::setup, task::Task, types::DirectoryId},
    async_trait::async_trait,
    gluesql::{
        core::ast_builder::Build,
        gluesql_git_storage::{GitStorage, StorageType},
        prelude::{FileStorage, Glue, Payload},
    },
    std::sync::mpsc::Sender,
};

pub struct Db {
    pub storage: Storage,
    pub root_id: DirectoryId,
    pub task_tx: Sender<Task>,
}

pub enum Storage {
    File(Glue<FileStorage>),
    Git(Glue<GitStorage>),
}

impl Db {
    pub async fn file(task_tx: Sender<Task>, path: &str) -> Result<Self> {
        let mut storage = FileStorage::new(path).map(Glue::new).map(Storage::File)?;

        let root_id = setup(&mut storage).await?;

        Ok(Self {
            storage,
            root_id,
            task_tx,
        })
    }

    pub async fn git(
        task_tx: Sender<Task>,
        path: &str,
        remote: String,
        branch: String,
    ) -> Result<Self> {
        let mut storage = GitStorage::open(path, StorageType::File)?;
        storage.set_remote(remote);
        storage.set_branch(branch);

        // Git cannot store empty directories. FileStorage still needs each
        // table directory when scanning a table with an existing schema file,
        // so recreate them before schema setup queries the tables.
        let notebook_path = std::path::Path::new(path);
        for table in ["Log", "Meta", "Directory", "Note"] {
            std::fs::create_dir_all(notebook_path.join(table)).map_err(|error| {
                Error::BackendError(format!(
                    "could not prepare Git notebook table directory {table}: {error}"
                ))
            })?;
        }

        let mut storage = Storage::Git(Glue::new(storage));
        let root_id = setup(&mut storage).await?;

        Ok(Self {
            storage,
            root_id,
            task_tx,
        })
    }

    pub fn pull(&mut self) -> Result<()> {
        if let Storage::Git(glue) = &mut self.storage {
            glue.storage.pull()?;
        }

        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        if let Storage::Git(glue) = &self.storage {
            let path = glue.storage.path.clone();
            let remote = glue.storage.remote.clone();
            let branch = glue.storage.branch.clone();

            let task = Task::GitSync {
                path,
                remote,
                branch,
            };

            self.task_tx
                .clone()
                .send(task)
                .map_err(|e| Error::BackendError(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait Execute
where
    Self: Sized,
{
    async fn execute(self, storage: &mut Storage) -> Result<Payload>;
}

#[async_trait]
impl<T: Build + Send> Execute for T
where
    Self: Sized,
{
    async fn execute(self, storage: &mut Storage) -> Result<Payload> {
        let statement = self.build()?;

        match storage {
            Storage::File(glue) => glue.execute_stmt(&statement).await,
            Storage::Git(glue) => glue.execute_stmt(&statement).await,
        }
        .map_err(Into::into)
    }
}
