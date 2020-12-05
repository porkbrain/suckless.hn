//! [`sqlite`][sqlite] database stores ids of top HN posts that are already
//! downloaded + some other metadata (timestamp of insertion, submission title,
//! url, which filters it passed).
//!
//! # Table `stories`
//! * `id` is the HN id
//! * `title` is the displayed HN title, always present
//! * `url` is either the article link or a link to the HN submission if
//!     the submission text was given instead of url
//! * `archive_url` is optional link to wayback machine snapshot or any other
//!     url to alternative source
//! * `created_at` is a [unix time][sqlite-time] of when we inserted into db
//!
//! # Table `story_filters`
//! * `story_id` is the HN id
//! * `amfg` is boolean set to 1 if filter flagged story
//! * `askhn` is boolean set to 1 if filter flagged story
//! * `bignews` is boolean set to 1 if filter flagged story
//! * `showhn` is boolean set to 1 if filter flagged story
//!
//! [sqlite]: https://github.com/rusqlite/rusqlite
//! [sqlite-time]: https://stackoverflow.com/q/200309/5093093#comment11501547_200329

use {
    fallible_iterator::FallibleIterator,
    rusqlite::{params, Connection, DatabaseName, NO_PARAMS},
    std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    },
};

use crate::{conf, filters::Filter, hn, prelude::*};

/// Creates sqlite connection to a file. If the file doesn't exist, creates
/// necessary tables.
///
/// If `BACKUPS_DIR` env var is set we backup the db.
pub fn conn(conf: &conf::Conf) -> Result<Connection> {
    let conn = Connection::open(&conf.sqlite_file)?;
    create_table_stories(&conn)?;
    create_table_story_filters(&conn)?;

    if let Some(backups_dir) = &conf.backups_dir {
        backup(&conn, backups_dir)?;
    }

    Ok(conn)
}

/// Synchronously inserts each story.
pub fn insert_stories(
    conn: &Connection,
    stories: impl IntoIterator<Item = Story>,
) -> Result<()> {
    log::debug!("Inserting new stories into db...");

    // TODO: Optimization is to insert stories in batch.
    for story in stories {
        insert_story(conn, story)?;
    }

    Ok(())
}

/// Inserts story ids associated with filters which it passed into the database.
/// Filters which flagged story are set to 1 (true), all other are defaulted to
/// 0 (false).
pub fn insert_filters(
    conn: &Connection,
    filters: &[StoryFilters],
) -> Result<()> {
    if filters.is_empty() {
        return Ok(());
    }

    let insert_stmts = filters.iter().map(|(id, filters)| {
        if filters.is_empty() {
            format!("INSERT INTO story_filters (story_id) VALUES ({});\n", id)
        } else {
            let filters_names: Vec<_> =
                filters.iter().map(|f| f.name()).collect();

            format!(
                "INSERT INTO story_filters (story_id, {}) VALUES ({}, {});\n",
                filters_names.as_slice().join(","),
                id,
                // 1 implies true (filter flagged story), rest defaults to 0
                ["1"].repeat(filters.len()).as_slice().join(",")
            )
        }
    });

    let sql = {
        let mut sql_builder = "BEGIN;\n".to_string();
        insert_stmts.for_each(|stmt| sql_builder.push_str(&stmt));
        sql_builder.push_str("COMMIT;");
        sql_builder
    };

    conn.execute_batch(&sql)?;

    Ok(())
}

/// Given list of HN story ids, discards the ones we already store in db.
pub fn only_new_stories(
    conn: &Connection,
    fetched_ids: Vec<StoryId>,
) -> Result<Vec<StoryId>> {
    if fetched_ids.is_empty() {
        log::warn!("No provided stories to deduplicate.");
        return Ok(vec![]);
    }

    let min_id = *fetched_ids.iter().min().unwrap(); // can't be empty
    let mut stmt =
        conn.prepare("SELECT id FROM stories WHERE id >= ?1 ORDER BY id ASC")?;
    let stored_ids: Vec<StoryId> = stmt
        .query(params![min_id])?
        .map(|r| r.get(0).map(|id: i64| id as StoryId))
        .collect()?;

    if stored_ids.is_empty() {
        log::debug!("No stored stories since story {}.", min_id);
        return Ok(fetched_ids);
    }

    // `stored_ids` sorted ASC
    let latest_stored_id = stored_ids[stored_ids.len() - 1]; // can't be empty

    let mut new_ids = fetched_ids;
    new_ids.retain(|id| {
        // `ids` newer than latest stored id will definitely be missing
        // bin search returns err if an id is not present
        *id > latest_stored_id || stored_ids.binary_search(id).is_err()
    });

    Ok(new_ids)
}

// Creates table `stories` if it doesn't exist yet. See the module docs for
// the fields description.
fn create_table_stories(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stories (
            id              INTEGER PRIMARY KEY,
            title           TEXT NOT NULL,
            url             TEXT NOT NULL,
            archive_url     TEXT,
            created_at      INTEGER(4)
        )",
        NO_PARAMS,
    )?;

    Ok(())
}

// Creates table `story_filters` if it doesn't exist yet. See the module docs
// for the fields description.
//
// TODO: How do we migrate to new filters?
fn create_table_story_filters(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS story_filters (
            story_id        INTEGER PRIMARY KEY,
            amfg            INTEGER(1) NOT NULL DEFAULT 0,
            askhn           INTEGER(1) NOT NULL DEFAULT 0,
            bignews         INTEGER(1) NOT NULL DEFAULT 0,
            showhn          INTEGER(1) NOT NULL DEFAULT 0
        )",
        NO_PARAMS,
    )?;

    Ok(())
}

// Creates a new backup file of the main database with current time in name.
fn backup(conn: &Connection, backups_dir: impl AsRef<Path>) -> Result<()> {
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let backup_file_path =
        backups_dir.as_ref().join(format!("{}.bak", unix_time));
    let progress_fn = None;

    log::info!("Backing up database into {:?}.", backup_file_path);
    conn.backup(DatabaseName::Main, backup_file_path, progress_fn)?;

    Ok(())
}

/// Inserts given story into the db. A submission with link will have url
/// pointing to the article, a text submission to the HN post.
fn insert_story(conn: &Connection, story: Story) -> Result<()> {
    let Story {
        id,
        title,
        kind,
        archive_url,
    } = story;
    log::trace!("Inserting story {}", id);

    let url = match kind {
        StoryKind::Url(url) => url,
        StoryKind::Text(_) => hn::submission_url(id),
    };

    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let mut stmt = conn.prepare(
        "INSERT INTO stories (id, title, url, archive_url, created_at) \
        VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    // sqlite doesn't support unsigned ints
    stmt.execute(params![id, title, url, archive_url, created_at as i64])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_only_new_stories() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        create_table_stories(&conn)?;

        assert_eq!(vec![1, 2, 3], only_new_stories(&conn, vec![1, 2, 3])?);

        let story1 = Story::random_url();
        let story1_id = story1.id;
        insert_story(&conn, story1)?;

        let story2 = Story::random_url();
        let story2_id = story2.id;
        insert_story(&conn, story2)?;

        assert_eq!(
            vec![1],
            only_new_stories(&conn, vec![1, story1_id, story2_id])?
        );

        Ok(())
    }

    #[test]
    fn it_inserts_story_filters() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        create_table_story_filters(&conn)?;

        insert_filters(
            &conn,
            &vec![
                (1, vec![FilterKind::AskHn, FilterKind::ShowHn]),
                (2, vec![FilterKind::FromMajorNewspaper]),
                (3, vec![]),
            ],
        )?;

        // TODO: select stories with modifiers

        Ok(())
    }
}
