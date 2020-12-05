mod archive;
mod conf;
mod db;
mod filters;
mod hn;
mod models;
mod prelude;

use rusqlite::Connection;

use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    log::info!("--- suckless.hn ---");

    let conf = conf::Conf::new();
    let conn = db::conn(&conf)?;

    let new_stories = get_new_stories(&conn, &conf).await?;

    log::info!("Applying Suckless Filters™");
    // TODO

    db::insert_stories(&conn, new_stories)?;

    log::info!("Generating html pages...");
    // TODO

    log::info!("Uploading all pages to s3...");
    // TODO

    Ok(())
}

// Puts together hn fetching, db queries and archive fetching.
async fn get_new_stories(
    conn: &Connection,
    conf: &conf::Conf,
) -> Result<Vec<Story>> {
    log::debug!("Fetching top stories list...");
    let mut top_stories = hn::fetch_top_stories().await?;
    top_stories.truncate(conf.top_stories_limit);
    let new_stories_ids = db::only_new_stories(&conn, top_stories)?;

    log::debug!("Fetching {} new stories...", new_stories_ids.len());
    let mut stories = hn::fetch_stories(&new_stories_ids).await?;

    log::debug!("Fetching snapshots for new stories...");
    archive::fetch_snapshots_for_stories(&mut stories).await?;

    Ok(stories)
}
