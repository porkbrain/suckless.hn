use super::*;

pub struct AskHn;
pub struct ShowHn;
pub struct FromMajorNewspaper;
pub struct MentionsBigTech;

impl Filter for AskHn {
    fn name(&self) -> &'static str {
        "askhn"
    }

    fn should_flag(&self, story: &Story) -> bool {
        story.title.starts_with("Ask HN")
    }
}

impl Filter for ShowHn {
    fn name(&self) -> &'static str {
        "showhn"
    }

    fn should_flag(&self, story: &Story) -> bool {
        story.title.starts_with("Show HN")
    }
}

impl Filter for FromMajorNewspaper {
    fn name(&self) -> &'static str {
        "bignews"
    }

    fn should_flag(&self, story: &Story) -> bool {
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
    fn name(&self) -> &'static str {
        "amfg"
    }

    fn should_flag(&self, story: &Story) -> bool {
        let t = &story.title;
        t.contains("Apple")
            || t.contains("Microsoft")
            || t.contains("Facebook")
            || t.contains("Google")
    }
}

// Kind of unfortunate but easier to work with a single enum type, but having
// impls on distinct structs.
impl Filter for FilterKind {
    fn name(&self) -> &'static str {
        match self {
            Self::AskHn => AskHn.name(),
            Self::ShowHn => ShowHn.name(),
            Self::FromMajorNewspaper => FromMajorNewspaper.name(),
            Self::MentionsBigTech => MentionsBigTech.name(),
        }
    }

    fn should_flag(&self, story: &Story) -> bool {
        match self {
            Self::AskHn => AskHn.should_flag(story),
            Self::ShowHn => ShowHn.should_flag(story),
            Self::FromMajorNewspaper => FromMajorNewspaper.should_flag(story),
            Self::MentionsBigTech => MentionsBigTech.should_flag(story),
        }
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
        assert!(FromMajorNewspaper.should_flag(&story));

        let story = Story::random_url();
        assert!(!FromMajorNewspaper.should_flag(&story));

        let story = Story::random_text();
        assert!(!FromMajorNewspaper.should_flag(&story));
    }
}
