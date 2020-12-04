//! Contains logic for storing and retrieving data from the sqlite database that
//! we store submissions into.

use {
    fallible_iterator::FallibleIterator,
    rusqlite::{params, Connection, NO_PARAMS},
    std::time::{SystemTime, UNIX_EPOCH},
};

use crate::{hn, prelude::*};

/// Creates table `stories` if it doesn't exist yet.
///
/// Fields:
/// * `id` is the HN id
/// * `title` is the displayed HN title, always present
/// * `url` is either the article link or a link to the HN submission if
///     the submission text was given instead of url
/// * `created_at` is a [unix time][sqlite-time] of when we inserted into db
///
/// [sqlite-time]: https://stackoverflow.com/q/200309/5093093#comment11501547_200329
pub fn create_table_stories(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stories (
            id              INTEGER PRIMARY KEY,
            title           TEXT NOT NULL,
            url             TEXT NOT NULL,
            created_at      INTEGER(4)
        )",
        NO_PARAMS,
    )?;

    Ok(())
}

pub fn insert_story(conn: &Connection, story: Story) -> Result<()> {
    let Story { id, title, kind } = story;
    log::trace!("Inserting story {}", id);

    let url = match kind {
        StoryKind::Url(url) => url,
        StoryKind::Text(_) => hn::submission_url(id),
    };

    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let mut stmt = conn.prepare(
        "INSERT INTO stories (id, title, url, created_at) \
        VALUES (?1, ?2, ?3, ?4)",
    )?;
    // sqlite doesn't support unsigned ints
    stmt.execute(params![id, title, url, created_at as i64])?;

    Ok(())
}

/// Given list of HN story ids, discards the ones we already store in db.
pub fn missing_stories(
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
    fn it_returns_missing_stories() -> Result<()> {
        let conn = Connection::open_in_memory()?;

        create_table_stories(&conn)?;

        assert_eq!(vec![1, 2, 3], missing_stories(&conn, vec![1, 2, 3])?);

        let story1 = Story::random_url();
        let story1_id = story1.id;
        insert_story(&conn, story1)?;

        let story2 = Story::random_url();
        let story2_id = story2.id;
        insert_story(&conn, story2)?;

        assert_eq!(
            vec![1],
            missing_stories(&conn, vec![1, story1_id, story2_id])?
        );

        Ok(())
    }
}
