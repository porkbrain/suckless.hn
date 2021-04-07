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
    rusqlite::{params, Connection, OptionalExtension},
    std::{
        convert::TryInto,
        time::{SystemTime, UNIX_EPOCH},
    },
};

use crate::{conf, filter::Filter, hn, prelude::*};

/// Creates sqlite connection to a file. If the file doesn't exist, creates
/// necessary tables.
pub fn conn(conf: &conf::Conf) -> Result<Connection> {
    let conn = Connection::open(&conf.sqlite_file)?;
    create_table_stories(&conn)?;
    create_table_story_filters(&conn)?;

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
        log::warn!("No filters to insert.");
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
    fetched_ids: &[StoryId],
) -> Result<Vec<StoryId>> {
    if fetched_ids.is_empty() {
        log::warn!("No stories to deduplicate.");
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
        return Ok(fetched_ids.to_vec());
    }

    // `stored_ids` sorted ASC
    let latest_stored_id = stored_ids[stored_ids.len() - 1]; // can't be empty

    let new_ids = fetched_ids
        .iter()
        .copied()
        .filter(|id| {
            // `ids` newer than latest stored id will definitely be missing
            // bin search returns err if an id is not present
            *id > latest_stored_id || stored_ids.binary_search(id).is_err()
        })
        .collect();

    Ok(new_ids)
}

/// Retrieves story along with the information about which filters flagged it.
pub fn select_story(
    conn: &Connection,
    story_id: StoryId,
) -> Result<Option<StoryWithFilters>> {
    // quite unfortunate that we have to have this gigantic return type
    // alternative is to implement [`From`] [`Row`] for [`StoryWithFilters`].
    let select_all_info = "
        SELECT s.id, s.title, s.url, s.archive_url, \
        sf.amfg, sf.askhn, sf.showhn, sf.bignews \
        FROM stories AS s \
        INNER JOIN story_filters AS sf ON s.id = sf.story_id \
        WHERE s.id = ? LIMIT 1
    ";
    type RowData = (
        StoryId,
        String,
        String,
        Option<String>,
        bool,
        bool,
        bool,
        bool,
    );

    let story = conn
        .query_row(select_all_info, params![story_id], |row| {
            let (id, title, url, archive_url, amfg, askhn, showhn, bignews): RowData
             = row.try_into()?;

            // only keeps filters which flagged the story
            let filters = [
                (amfg, FilterKind::BigTech),
                (askhn, FilterKind::AskHn),
                (showhn, FilterKind::ShowHn),
                (bignews, FilterKind::LargeNewspaper),
            ]
            .iter()
            .copied()
            .filter_map(
                |(applies, filter)| if applies { Some(filter) } else { None },
            )
            .collect();

            Ok(StoryWithFilters {
                id,
                title,
                url,
                archive_url,
                filters,
            })
        })
        .optional()?;

    Ok(story)
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
        [],
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
            showhn          INTEGER(1) NOT NULL DEFAULT 0,
            FOREIGN KEY(story_id) REFERENCES stories(id)
        )",
        [],
    )?;

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
pub mod tests {
    use super::*;

    pub fn test_conn() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        create_table_stories(&conn)?;
        create_table_story_filters(&conn)?;
        Ok(conn)
    }

    #[test]
    fn it_returns_only_new_stories() -> Result<()> {
        let conn = test_conn()?;

        assert_eq!(vec![1, 2, 3], only_new_stories(&conn, &vec![1, 2, 3])?);

        let story1 = Story::random_url();
        let story1_id = story1.id;
        insert_story(&conn, story1)?;

        let story2 = Story::random_url();
        let story2_id = story2.id;
        insert_story(&conn, story2)?;

        assert_eq!(
            vec![1],
            only_new_stories(&conn, &vec![1, story1_id, story2_id])?
        );

        Ok(())
    }

    #[test]
    fn it_selects_story() -> Result<()> {
        let conn = test_conn()?;

        let stories = &[
            (
                Story::random_url(),
                vec![FilterKind::AskHn, FilterKind::ShowHn],
            ),
            (Story::random_url(), vec![FilterKind::LargeNewspaper]),
            (Story::random_url(), vec![]),
        ];
        insert_test_data(&conn, stories)?;

        // for each story, gets it from db and asserts it equals the test data
        for (story, filters) in stories {
            let db_story = select_story(&conn, story.id)?.unwrap();

            assert_eq!(story.id, db_story.id);
            assert_eq!(&story.title, &db_story.title);

            assert_eq!(filters.len(), db_story.filters.len());
            for filter in filters {
                assert!(db_story.filters.contains(filter));
            }
        }

        Ok(())
    }

    /// Inserts given stories + filters to the database.
    pub fn insert_test_data(
        conn: &Connection,
        stories: &[(Story, Vec<FilterKind>)],
    ) -> Result<()> {
        insert_stories(
            &conn,
            stories
                .iter()
                .map(|(story, _)| story.clone())
                .collect::<Vec<_>>(),
        )?;

        insert_filters(
            &conn,
            &stories
                .iter()
                .map(|(story, filters)| (story.id, filters.clone()))
                .collect::<Vec<_>>(),
        )?;

        Ok(())
    }
}
