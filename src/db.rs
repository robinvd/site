use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Error, bail};
use crossbeam_channel::Sender;
use dashmap::{DashMap, Entry};
use notify_debouncer_mini::{
    DebounceEventResult, Debouncer, new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
};
use salsa::Accumulator;

#[salsa::db]
pub trait Db: salsa::Database {
    fn input(&self, path: PathBuf) -> Result<File, Error>;
    fn dir(&self, path: PathBuf) -> Result<Dir, Error>;
}

#[salsa::accumulator]
pub struct Diagnostic(pub String);

impl Diagnostic {
    pub fn push_error(db: &dyn Db, file: &Path, error: Error) {
        Diagnostic(format!(
            "Error in file {}: {:?}\n",
            file.file_name()
                .unwrap_or_else(|| "<unknown>".as_ref())
                .to_string_lossy(),
            error,
        ))
        .accumulate(db);
    }
}

#[salsa::input(debug)]
pub struct File {
    pub path: PathBuf,
    #[return_ref]
    pub text: Vec<u8>,
}

#[salsa::input(debug)]
pub struct Dir {
    pub path: PathBuf,
    #[return_ref]
    pub items: Vec<PathBuf>,
}

#[derive(Clone, Copy)]
pub enum FileItem {
    File(File),
    Dir(Dir),
}

#[salsa::interned(debug)]
pub struct Tag<'db> {
    #[return_ref]
    pub name: String,
}

#[salsa::db]
#[derive(Clone)]
pub struct BlogDatabase {
    storage: salsa::Storage<Self>,

    // The logs are only used for testing and demonstrating reuse:
    pub logs: Arc<Mutex<Vec<String>>>,

    pub files: DashMap<PathBuf, FileItem>,
    pub file_watcher: Option<Arc<Mutex<Debouncer<RecommendedWatcher>>>>,
}

impl BlogDatabase {
    pub fn new_watch(tx: Sender<DebounceEventResult>) -> Self {
        Self {
            storage: Default::default(),
            logs: Default::default(),
            files: DashMap::new(),
            file_watcher: Some(Arc::new(Mutex::new(
                new_debouncer(Duration::from_secs(1), tx).unwrap(),
            ))),
        }
    }

    pub fn new() -> Self {
        Self {
            storage: Default::default(),
            logs: Default::default(),
            files: DashMap::new(),
            file_watcher: None,
        }
    }
}

#[salsa::db]
impl salsa::Database for BlogDatabase {
    fn salsa_event(&self, event: &dyn Fn() -> salsa::Event) {
        let event = event();
        let logs = &mut *self.logs.lock().unwrap();
        // only log interesting events
        if let salsa::EventKind::WillExecute { .. } = event.kind {
            logs.push(format!("Event: {event:?}"));
        }
    }
}

#[salsa::db]
impl Db for BlogDatabase {
    fn input(&self, path: PathBuf) -> Result<File, Error> {
        let path = path
            .canonicalize()
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Ok(match self.files.entry(path.clone()) {
            // If the file already exists in our cache then just return it.
            Entry::Occupied(entry) => match *entry.get() {
                FileItem::File(f) => f,
                FileItem::Dir(_) => bail!("Path for `input` is not a file; {}", path.display()),
            },
            // If we haven't read this file yet set up the watch, read the
            // contents, store it in the cache, and return it.
            Entry::Vacant(entry) => {
                let contents = std::fs::read(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                let file = File::new(self, path.to_path_buf(), contents);
                // Set up the watch before reading the contents to try to avoid
                // race conditions.
                if let Some(file_watcher) = self.file_watcher.as_ref() {
                    let watcher = &mut *file_watcher.lock().unwrap();
                    watcher
                        .watcher()
                        .watch(&path, RecursiveMode::NonRecursive)
                        .unwrap();
                    entry.insert(FileItem::File(file));
                }
                file
            }
        })
    }

    fn dir(&self, path: PathBuf) -> Result<Dir, Error> {
        let path = path
            .canonicalize()
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Ok(match self.files.entry(path.clone()) {
            // If the file already exists in our cache then just return it.
            Entry::Occupied(entry) => match *entry.get() {
                FileItem::File(_) => bail!("Path for `dir` is not a file; {}", path.display()),
                FileItem::Dir(d) => d,
            },
            // If we haven't read this file yet set up the watch, read the
            // contents, store it in the cache, and return it.
            Entry::Vacant(entry) => {
                let contents = fs::read_dir(&path)?
                    .map(|entry| Ok(entry?.path().to_owned()))
                    .collect::<Result<Vec<_>, Error>>()?;
                let dir = Dir::new(self, path.to_path_buf(), contents);
                // Set up the watch before reading the contents to try to avoid
                // race conditions.
                if let Some(file_watcher) = self.file_watcher.as_ref() {
                    let watcher = &mut *file_watcher.lock().unwrap();
                    watcher
                        .watcher()
                        .watch(&path, RecursiveMode::NonRecursive)
                        .unwrap();
                    entry.insert(FileItem::Dir(dir));
                }
                dir
            }
        })
    }
}
