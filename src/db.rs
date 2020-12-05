//! Contains logic for storing and retrieving data from the sqlite database that
//! we store stories in. That means a single `stories` table.
//!
//! Fields of `stories`:
//! * `id` is the HN id
//! * `title` is the displayed HN title, always present
//! * `url` is either the article link or a link to the HN submission if
//!     the submission text was given instead of url
//! * `archive_url` is optional link to wayback machine snapshot or any other
//!     url to alternative source
//! * `created_at` is a [unix time][sqlite-time] of when we inserted into db
//!
//! [sqlite-time]: https://stackoverflow.com/q/200309/5093093#comment11501547_200329

use {
    fallible_iterator::FallibleIterator,
    rusqlite::{params, Connection, DatabaseName, NO_PARAMS},
    std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    },
};

use crate::{hn, prelude::*};

/// Creates table `stories` if it doesn't exist yet. See the module docs for
/// the fields description.
pub fn create_table_stories(conn: &Connection) -> Result<()> {
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

/// Creates a new backup file of the main database with current time in name.
pub fn backup(conn: &Connection, backups_dir: impl AsRef<Path>) -> Result<()> {
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let backup_file_path =
        backups_dir.as_ref().join(format!("{}.bak", unix_time));
    let progress_fn = None;

    log::info!("Backing up database into {:?}.", backup_file_path);
    conn.backup(DatabaseName::Main, backup_file_path, progress_fn)?;

    Ok(())
}

/// Synchronously inserts each story.
pub fn insert_stories(
    conn: &Connection,
    stories: impl IntoIterator<Item = Story>,
) -> Result<()> {
    // TODO: Optimization is to insert stories in batch.
    for story in stories {
        insert_story(conn, story)?;
    }

    Ok(())
}

/// Inserts given story into the db. A submission with link will have url
/// pointing to the article, a text submission to the HN post.
pub fn insert_story(conn: &Connection, story: Story) -> Result<()> {
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

/// Given list of HN story ids, discards the ones we already store in db.
pub fn new_stories(
    conn: &Connection,
    ids: Vec<StoryId>,
) -> Result<Vec<StoryId>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let min_id = *ids.iter().min().unwrap(); // can't be empty
    let mut stmt =
        conn.prepare("SELECT id FROM stories WHERE id >= ?1 ORDER BY id ASC")?;
    let stored_ids: Vec<StoryId> = stmt
        .query(params![min_id])?
        .map(|r| r.get(0).map(|id: i64| id as StoryId))
        .collect()?;

    if stored_ids.is_empty() {
        log::debug!("No stored stories since story {}.", min_id);
        return Ok(ids);
    }

    // `stored_ids` sorted ASC
    let latest_stored_id = stored_ids[stored_ids.len() - 1]; // can't be empty

    let mut ids = ids;
    ids.retain(|id| {
        // `ids` newer than latest stored id will definitely be missing
        // bin search returns err if an id is not present
        *id > latest_stored_id || stored_ids.binary_search(id).is_err()
    });

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_new_stories() -> Result<()> {
        let conn = Connection::open_in_memory()?;

        create_table_stories(&conn)?;

        assert_eq!(vec![1, 2, 3], new_stories(&conn, vec![1, 2, 3])?);

        let story1 = Story::random_url();
        let story1_id = story1.id;
        insert_story(&conn, story1)?;

        let story2 = Story::random_url();
        let story2_id = story2.id;
        insert_story(&conn, story2)?;

        assert_eq!(vec![1], new_stories(&conn, vec![1, story1_id, story2_id])?);

        Ok(())
    }
}
