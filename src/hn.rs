//! This module contains logic which polls HackerNews APIs.

use crate::prelude::*;

const FIREBASE_API: &str = "https://hacker-news.firebaseio.com/v0";

/// Given id, returns the url where the submission can be viewed on HN.
pub fn submission_url(id: StoryId) -> String {
    format!("https://news.ycombinator.com/item?id={}", id)
}

/// Polls HN Firebase JSON APIs and grabs [top stories][hn-topstories].
///
/// The stories returned from the APIs are sorted by their position on the HN
/// front page (ASC).
///
/// [hn-topstories]: https://hacker-news.firebaseio.com/v0/topstories.json
pub async fn top_stories() -> Result<Vec<StoryId>> {
    let url = format!("{}/topstories.json", FIREBASE_API);
    let stories: Vec<StoryId> = reqwest::get(&url).await?.json().await?;

    Ok(stories)
}

/// Return [single story][hn-item] from HN Firebase APIs by querying endpoint
/// `https://hacker-news.firebaseio.com/v0/item/${STORY_ID}.json`.
///
/// [hn-item]: https://github.com/HackerNews/API#items
pub async fn story(id: StoryId) -> Result<Story> {
    let url = format!("{}/item/{}.json", FIREBASE_API, id);
    let story = reqwest::get(&url).await?.json().await?;

    Ok(story)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_fetches_top_stories() -> Result<()> {
        let stories = top_stories().await?;
        assert_ne!(0, stories.len());

        Ok(())
    }

    #[tokio::test]
    async fn it_fetches_ask_hn() -> Result<()> {
        // https://news.ycombinator.com/item?id=23366546
        let story_id = 23366546;
        let story = story(story_id).await?;

        assert_eq!(
            "Ask HN: \
            Am I the longest-serving programmer – 57 years and counting?",
            &story.title
        );

        match &story.kind {
            StoryKind::Text(text) => {
                assert_eq!("In May of 1963", &text[0..14])
            }
            _ => panic!("Expected Ask HN story with text"),
        };

        Ok(())
    }

    #[tokio::test]
    async fn it_fetches_url_submission() -> Result<()> {
        // https://news.ycombinator.com/item?id=25300310
        let story_id = 25300310;
        let story = story(story_id).await?;

        assert_eq!("Bit Twiddling Hacks", &story.title);

        match &story.kind {
            StoryKind::Url(url) => {
                assert_eq!(
                    "https://graphics.stanford.edu/~seander/bithacks.html",
                    url
                );
            }
            _ => panic!("Expected Ask HN story with text"),
        };

        Ok(())
    }
}
