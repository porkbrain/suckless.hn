use serde::{Deserialize, Serialize};

pub type StoryId = i64;
pub type StoryFilters = (StoryId, Vec<FilterKind>);

/// Supported filters, for specifics see [`filters::impls`] module.
// TODO(https://github.com/rust-lang/rust/issues/27747): Impl [`std::slice::Join`]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FilterKind {
    AskHn,
    ShowHn,
    FromMajorNewspaper,
    MentionsBigTech,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(Clone, Debug, PartialEq))]
pub struct Story {
    pub id: StoryId,
    pub title: String,
    /// Optional url to wayback machine.
    pub archive_url: Option<String>,
    /// Flattening the kind allows us to use enum instead of two mutually
    /// exclusive options.
    #[serde(flatten)]
    pub kind: StoryKind,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(Clone, Debug, PartialEq))]
pub enum StoryKind {
    // [`Story`] will have property "url".
    Url(String),
    // [`Story`] will have property "text".
    Text(String),
}

/// Story information which we retrieve from the database. A join query on both
/// `stories` and `story_filters` tables.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryWithFilters {
    pub id: StoryId,
    pub title: String,
    pub url: String,
    pub archive_url: Option<String>,
    pub filters: Vec<FilterKind>,
}

/// Determines whether we are interested in stories matching or not matching
/// given filter.
pub enum Modifier {
    With(FilterKind),
    Without(FilterKind),
}

#[cfg(test)]
mod tests {
    //! Implement method factory methods used by other tests.

    use {names::Generator, rand::random};

    use super::*;

    impl Story {
        pub fn random_url() -> Self {
            let mut gen = Generator::default();
            Self {
                id: random::<i64>().abs(),
                title: gen.next().unwrap(),
                archive_url: None,
                kind: StoryKind::Url(format!(
                    "https://{}.com/random/{}",
                    gen.next().unwrap(),
                    gen.next().unwrap()
                )),
            }
        }

        pub fn random_text() -> Self {
            let mut gen = Generator::default();
            Self {
                id: random::<i64>().abs(),
                title: gen.next().unwrap(),
                archive_url: None,
                kind: StoryKind::Text(gen.next().unwrap()),
            }
        }
    }
}
