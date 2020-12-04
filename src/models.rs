use serde::Deserialize;

pub type StoryId = i64;

#[derive(Deserialize, Debug)]
pub struct Story {
    pub id: StoryId,
    pub title: String,
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
    use {names::Generator, rand::random};

    use super::*;

    // Implement story factory methods.
    impl Story {
        pub fn random_url() -> Self {
            let mut gen = Generator::default();
            Self {
                id: random::<i64>().abs(),
                title: gen.next().unwrap(),
                kind: StoryKind::Url(format!(
                    "https://example.com/{}",
                    gen.next().unwrap()
                )),
            }
        }

        pub fn random_text() -> Self {
            let mut gen = Generator::default();
            Self {
                id: random::<i64>().abs(),
                title: gen.next().unwrap(),
                kind: StoryKind::Text(gen.next().unwrap()),
            }
        }
    }
}
