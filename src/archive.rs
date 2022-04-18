//! Along with the article link, suckless.hn provides Wayback machine archive
//! link to the submission.
//!
//! * TODO(https://github.com/bausano/suckless.hn/issues/2): Submit a url if
//!     snapshot doesn't exist yet.
//! * TODO(https://github.com/bausano/suckless.hn/issues/1): Get 'timestamp'
//!     property and ignore snapshots older than a month.

use serde::Deserialize;

use crate::prelude::*;

/// Downloads snapshot from Wayback machine if one exists and assigns it to the
/// model.
pub async fn fetch_snapshots_for_stories(stories: &mut [Story]) -> Result<()> {
    // we run this sequentially because wayback machine APIs are quick to
    // throttle concurrent requests
    for story in stories {
        // only requests snapshots for urls, not for text
        if let StoryKind::Url(url) = &story.kind {
            match fetch_snapshot(url).await {
                Ok(snapshot) => story.archive_url = snapshot,
                Err(e) => {
                    log::warn!("Cannot check snapshot for {}: {}", url, e);
                }
            }
        }
    }

    Ok(())
}

// Checks if given url has a snapshot available.
async fn fetch_snapshot(url: &str) -> Result<Option<String>> {
    // {
    //     "archived_snapshots": {
    //         "closest": { "url": "..." }
    //     }
    // }
    #[derive(Deserialize)]
    struct WaybackResponse {
        archived_snapshots: ClosestSnapshot,
    }
    #[derive(Deserialize)]
    struct ClosestSnapshot {
        closest: Option<Url>,
    }
    #[derive(Deserialize)]
    struct Url {
        url: String,
    }

    let url = format!("http://archive.org/wayback/available?url={}", url);
    let resp: WaybackResponse = reqwest::get(&url).await?.json().await?;

    Ok(resp.archived_snapshots.closest.map(|snapshot| snapshot.url))
}

#[cfg(test)]
mod tests {
    //! These sets must be ran sequentially, hence they're in a single test
    //! case. Not running them sequentially causes some weird failures.

    use super::*;

    #[tokio::test]
    async fn it_fetches_snapshots() -> Result<()> {
        env_logger::init();

        // single existing snapshot
        let snapshot = fetch_snapshot("https://porkbrain.com").await?;
        assert_ne!(None, snapshot, "Expected snapshot");

        // single non existing snapshot
        let snapshot =
            fetch_snapshot("https://porkbrain.com/non-existent").await?;
        assert_eq!(None, snapshot, "Didn't expect snapshot");

        // multiple snapshots
        let mut porkbrain = Story::random_url();
        porkbrain.kind = StoryKind::Url("https://porkbrain.com".to_string());
        let mut stories =
            vec![porkbrain, Story::random_url(), Story::random_text()];

        fetch_snapshots_for_stories(&mut stories).await?;

        assert_ne!(None, stories[0].archive_url);
        assert_eq!(None, stories[1].archive_url);
        assert_eq!(None, stories[2].archive_url);

        Ok(())
    }
}
