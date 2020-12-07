pub mod impls;

pub use impls::*;

use {
    serde::{Deserialize, Serialize},
    std::collections::HashSet,
};

pub type StoryId = i64;
pub type StoryFilters = (StoryId, Vec<FilterKind>);

/// Supported filters, for specifics see [`filter::impls`] module.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum FilterKind {
    AskHn,
    ShowHn,
    LargeNewspaper,
    BigTech,
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
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoryWithFilters {
    pub id: StoryId,
    pub title: String,
    pub url: String,
    pub archive_url: Option<String>,
    pub filters: HashSet<FilterKind>,
}

/// Determines whether we are interested in stories matching or not matching
/// given filter.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Modifier {
    With(FilterKind),
    Without(FilterKind),
}

/// We support different themes.
#[derive(Copy, Clone)]
pub enum Theme {
    Dark,
    Light,
}

#[cfg(test)]
mod tests {
    //! Implement method factory methods used by other tests.

    use {names::Generator, rand::random};

    use super::*;

    impl StoryWithFilters {
        pub fn random(filters: Vec<FilterKind>) -> Self {
            let Story {
                id,
                title,
                archive_url,
                ..
            } = Story::random_url();

            Self {
                id,
                title,
                url: random_url(),
                archive_url,
                filters: filters.into_iter().collect(),
            }
        }
    }

    impl Story {
        pub fn random_url() -> Self {
            let mut gen = Generator::default();
            Self {
                id: random::<i64>().abs(),
                title: gen.next().unwrap(),
                archive_url: None,
                kind: StoryKind::Url(random_url()),
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

    fn random_url() -> String {
        let mut gen = Generator::default();
        format!(
            "https://{}.com/random/{}",
            gen.next().unwrap(),
            gen.next().unwrap()
        )
    }
}
