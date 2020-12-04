//! Along with the article link, suckless.hn provides Wayback machine archive
//! link to the submission.
//!
//! * TODO(https://github.com/bausano/suckless.hn/issues/2): Submit a url if
//!     snapshot doesn't exist yet.
//! * TODO(https://github.com/bausano/suckless.hn/issues/1): Get 'timestamp'
//!     property and ignore snapshots older than a month.

use serde::Deserialize;

use crate::prelude::*;

// Checks if given url has a snapshot available.
async fn fetch_snapshot(url: &str) -> Result<Option<String>> {
    // {
    //     "archived_snapshots": {
    //         "closest": { "url": "..." }
    //     }
    // }
    #[derive(Deserialize)]
    struct WaybackResponse {
        archived_snapshots: Option<ClosestSnapshot>,
    }
    #[derive(Deserialize)]
    struct ClosestSnapshot {
        closest: Url,
    }
    #[derive(Deserialize)]
    struct Url {
        url: String,
    }

    let url = format!("http://archive.org/wayback/available?url={}", url);
    let resp: WaybackResponse = reqwest::get(&url).await?.json().await?;

    Ok(resp.archived_snapshots.map(|snapshot| snapshot.closest.url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_fetches_snapshot() -> Result<()> {
        let snapshot = fetch_snapshot("https://porkbrain.com").await?;
        assert_ne!(None, snapshot);

        Ok(())
    }
}
