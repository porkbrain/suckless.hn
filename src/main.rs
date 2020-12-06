mod archive;
mod conf;
mod db;
mod filter;
mod hn;
mod html;
mod models;
mod prelude;

use rusqlite::Connection;

use {filter::page, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    log::info!("--- suckless.hn ---");

    let conf = conf::Conf::new();
    let conn = db::conn(&conf)?;

    log::info!("Fetching top stories list...");
    let top_stories = hn::fetch_top_stories().await?;
    let new_stories =
        fetch_new_stories(&conn, &top_stories, conf.new_stories_limit).await?;

    log::info!("Applying Suckless Filters™...");
    let new_stories_filters = filter::for_stories(&new_stories);

    db::insert_stories(&conn, new_stories)?;
    db::insert_filters(&conn, &new_stories_filters)?;

    log::info!("Generating html pages and uploading to S3...");
    let engine = html::Template::new()?;
    let pages = page::populate(&conn, top_stories, conf.stories_per_page);

    let jobs: Vec<_> = pages
        .into_iter()
        .map(|page| page.upload(&conf, &engine))
        .collect();
    let results: Vec<Result<()>> = futures::future::join_all(jobs).await;

    for error in results.into_iter().filter_map(|r| r.err()) {
        log::error!("Cannot upload page: {}", error);
    }

    Ok(())
}

// Puts together hn fetching, db queries and archive fetching.
async fn fetch_new_stories(
    conn: &Connection,
    top_stories: &[StoryId],
    new_stories_limit: usize,
) -> Result<Vec<Story>> {
    log::debug!(
        "Checking how many out of the {} top stories are already stored.",
        top_stories.len()
    );
    let mut new_stories_ids = db::only_new_stories(&conn, top_stories)?;
    new_stories_ids.truncate(new_stories_limit);

    log::debug!("Fetching {} new stories...", new_stories_ids.len());
    let mut stories = hn::fetch_stories(&new_stories_ids).await?;

    log::debug!("Fetching snapshots for new stories...");
    archive::fetch_snapshots_for_stories(&mut stories).await?;

    Ok(stories)
}
