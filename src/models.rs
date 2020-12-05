use serde::Deserialize;

pub type StoryId = i64;

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum StoryKind {
    // [`Story`] will have property "url".
    Url(String),
    // [`Story`] will have property "text".
    Text(String),
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
