mod archive;
mod conf;
mod db;
mod hn;
mod models;
mod prelude;

use {
    rusqlite::{Connection, DatabaseName},
    std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    },
};

use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("--- suckless.hn ---");
    let conf = conf::Conf::new();
    let sqlite = Connection::open(&conf.sqlite_file)?;

    if let Some(backups_dir) = &conf.backups_dir {
        backup_db(&sqlite, backups_dir)?;
    }

    // let top_stories = hn::top_stories(conf.top_stories_limit).await?;

    // look into db for their ids to find which are missing and which aren't

    Ok(())
}

// Create a new backup file of the main database with current time in name.
fn backup_db(sqlite: &Connection, backups_dir: impl AsRef<Path>) -> Result<()> {
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let backup_file_path =
        backups_dir.as_ref().join(format!("{}.bak", unix_time));
    let progress_fn = None;

    log::info!("Backing up database into {:?}.", backup_file_path);
    sqlite.backup(DatabaseName::Main, backup_file_path, progress_fn)?;

    Ok(())
}
