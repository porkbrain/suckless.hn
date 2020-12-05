mod archive;
mod conf;
mod db;
mod hn;
mod models;
mod prelude;

use { rusqlite::Connection};

use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("--- suckless.hn ---");
    let conf = conf::Conf::new();
    let conn = Connection::open(&conf.sqlite_file)?;
    db::create_table_stories(&conn)?;

    if let Some(backups_dir) = &conf.backups_dir {
        db::backup(&conn, backups_dir)?;
    }

    let new_stories = {
        log::info!("Fetching top stories list...");
        let top_stories = hn::fetch_top_stories().await?;
        let mut new_stories_ids = db::new_stories(&conn, top_stories)?;
        new_stories_ids.truncate(conf.top_stories_limit);
        log::info!("Fetching {} new stories...", new_stories_ids.len());
        let mut stories = hn::fetch_stories(&new_stories_ids).await?;
        log::info!("Fetching snapshots for new stories...");
        archive::fetch_snapshots_for_stories(&mut stories).await?;
        stories
    };

    log::info!("Applying Suckless Filters™");
    // TODO

    log::info!("Inserting new stories into db...");
    db::insert_stories(&conn, new_stories)?;

    log::info!("Generating html pages...");
    // TODO

    log::info!("Uploading all pages to s3...");
    // TODO

    Ok(())
}
