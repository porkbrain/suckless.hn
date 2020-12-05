//! The raison d'être of this codebase, the Suckless Filters™.
//!
//! Given a story, filter will decide based on the story content whether to flag
//! it. This information is then written to the database.

mod impls;

use {lazy_static::lazy_static, regex::Regex};

use crate::prelude::*;

pub trait Filter {
    /// Name of the filter group. For now all filter groups are hard coded.
    /// We use name instead of [`std::fmt::Display`] because the impl is less
    /// code and we can use 'static.
    fn name(&self) -> &'static str;

    /// Does the filter apply to the given story?
    fn should_flag(&self, story: &Story) -> bool;
}

// IMPORTANT: This needs to be sorted based on name.
const FILTERS: &[FilterKind] = &[
    FilterKind::MentionsBigTech,
    FilterKind::AskHn,
    FilterKind::FromMajorNewspaper,
    FilterKind::ShowHn,
];

/// Given stories, returns a list of filters which flagged each story.
/// The output vector is of the same size as the input.
pub fn for_stories(stories: &[Story]) -> Vec<StoryFilters> {
    stories
        .iter()
        .map(|story| {
            let story_filters = FILTERS
                .iter()
                .copied()
                .filter(|f| f.should_flag(story))
                .collect();

            (story.id, story_filters)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_sorted_list_of_filters() {
        let mut filters = FILTERS.to_vec();
        filters.sort_by(|a, b| a.name().cmp(b.name()));
        assert_eq!(filters.as_slice(), FILTERS);
        // https://github.com/rust-lang/rust/issues/53485
        // debug_assert!(FILTERS.is_sorted_by(|a, b| a.name().cmp(b.name())));
    }

    #[test]
    fn it_picks_filters_for_stories() {
        let bbc_google_story = {
            let mut story = Story::random_url();
            story.kind = StoryKind::Url("https://bbc.com".to_string());
            story.title = "Pure Google mate".to_string();
            story
        };
        let bbc_google_story_id = bbc_google_story.id;

        let ask_hn_story = {
            let mut story = Story::random_url();
            story.title = "Ask HN: Hello".to_string();
            story
        };
        let ask_hn_story_id = ask_hn_story.id;

        let stories = &[bbc_google_story, ask_hn_story, Story::random_text()];

        let filters = for_stories(stories);
        assert_eq!(3, filters.len());
        assert_eq!(
            (
                bbc_google_story_id,
                vec![
                    FilterKind::MentionsBigTech,
                    FilterKind::FromMajorNewspaper
                ]
            ),
            filters[0]
        );
        assert_eq!((ask_hn_story_id, vec![FilterKind::AskHn]), filters[1]);
        assert!(filters[2].1.is_empty());
    }
}
