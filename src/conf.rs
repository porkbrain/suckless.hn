use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Conf {
    /// Where should the main database be stored.
    pub sqlite_file: PathBuf,
    /// If provided, we run a db backup before we start downloading new items.
    pub backups_dir: Option<PathBuf>,
    /// How many new stories should we fetch from the HNs APIs and display on
    /// the suckless.hn front page.
    pub new_stories_limit: usize,
}

impl Conf {
    /// Creates config from env vars.
    ///
    /// # Panic
    /// If any mandatory var is missing or malformed.
    pub fn new() -> Self {
        let sqlite_file = env::var(vars::SQLITE_FILE)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                panic!("Missing env var {}.", vars::SQLITE_FILE)
            });
        log::debug!("{}={:?}", vars::SQLITE_FILE, sqlite_file);

        let backups_dir = env::var(vars::BACKUPS_DIR)
            .map(PathBuf::from)
            .map(Some)
            .unwrap_or(None);
        log::debug!("{}={:?}", vars::BACKUPS_DIR, backups_dir);
        if let Some(backups_dir) = &backups_dir {
            assert!(backups_dir.is_dir(), "Backups dir must exist");
        }

        // swallows parsing errors but it's ok for our use case
        let new_stories_limit = env::var(vars::NEW_STORIES_LIMIT)
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(defaults::NEW_STORIES_LIMIT);
        log::debug!("{}={:?}", vars::NEW_STORIES_LIMIT, new_stories_limit);

        Self {
            sqlite_file,
            backups_dir,
            new_stories_limit,
        }
    }
}

mod vars {
    pub const SQLITE_FILE: &str = "SQLITE_FILE";
    pub const BACKUPS_DIR: &str = "BACKUPS_DIR"; // opt
    pub const NEW_STORIES_LIMIT: &str = "NEW_STORIES_LIMIT"; // opt
}

mod defaults {
    pub const NEW_STORIES_LIMIT: usize = 30;
}
