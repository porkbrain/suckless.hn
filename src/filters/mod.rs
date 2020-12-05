//! The raison d'être of this codebase, the Suckless Filters™.
//!
//! Given a story, filter will decide based on the story content whether to flag
//! it. This information is then written to the database.

use {lazy_static::lazy_static, regex::Regex};

use crate::prelude::*;

pub trait Filter {
    /// Name of the filter group. For now all filter groups are hard coded.
    /// We use name instead of [`std::fmt::Display`] because the impl is less
    /// code and we can use 'static.
    fn name() -> &'static str;

    /// Does the filter group apply to the given story?
    fn applies(story: &Story) -> bool;
}

pub struct AskHn;
pub struct ShowHn;
pub struct FromMajorNewspaper;
pub struct MentionsBigTech;

impl Filter for AskHn {
    fn name() -> &'static str {
        "askhn"
    }

    fn applies(story: &Story) -> bool {
        story.title.starts_with("Ask HN")
    }
}

impl Filter for ShowHn {
    fn name() -> &'static str {
        "showhn"
    }

    fn applies(story: &Story) -> bool {
        story.title.starts_with("Show HN")
    }
}

impl Filter for FromMajorNewspaper {
    fn name() -> &'static str {
        "bignews"
    }

    fn applies(story: &Story) -> bool {
        lazy_static! {
            static ref NEWSPAPER_WEBSITE: Regex = Regex::new(concat!(
                "https?://", // doesn't have to be tls
                "(?:www\\.)?", // can start with www
                "(?:", // start non-capturing group of websites
                "bbc\\.com|",
                "wsj\\.com|",
                "bloomberg\\.com|",
                "vice\\.com|",
                "theguardian\\.com|",
                "cnbc\\.com|",
                "forbes\\.com",
                ")"
            )).expect("Invalid newspaper website regex");
        }

        match &story.kind {
            StoryKind::Url(url) => NEWSPAPER_WEBSITE.is_match(&url),
            StoryKind::Text(_) => false,
        }
    }
}

impl Filter for MentionsBigTech {
    fn name() -> &'static str {
        "amfg"
    }

    fn applies(story: &Story) -> bool {
        let t = &story.title;
        t.contains("Apple")
            || t.contains("Microsoft")
            || t.contains("Facebook")
            || t.contains("Google")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_match_newspapers() {
        let news = "https://www.wsj.com/articles/reddit-claims-52-million-daily-users-revealing-a-key-figure-for-social-media-platforms-11606822200";
        let mut story = Story::random_url();
        story.kind = StoryKind::Url(news.to_string());
        assert!(FromMajorNewspaper::applies(&story));

        let story = Story::random_url();
        assert!(!FromMajorNewspaper::applies(&story));

        let story = Story::random_text();
        assert!(!FromMajorNewspaper::applies(&story));
    }
}
