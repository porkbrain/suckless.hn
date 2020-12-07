//! We set up pages by iterating over top stories and querying them from the
//! database. For each page we have defined we keep a reference to stories which
//! are flagged.

use {
    rusqlite::Connection,
    std::{io, rc::Rc},
    tokio::fs,
};

use crate::{conf, db, html::Template, prelude::*};

#[derive(Debug)]
pub struct Page {
    // Modifiers are hard coded.
    modifiers: &'static [Modifier],
    // Name is generated from the modifiers.
    name: String,
    stories: Vec<Rc<StoryWithFilters>>,
}

impl Page {
    /// All stories without any filter.
    pub fn all() -> Self {
        Self::new(&[])
    }

    pub fn ask_hn() -> Self {
        Self::new(&[Modifier::With(FilterKind::AskHn)])
    }

    pub fn no_ask_hn() -> Self {
        Self::new(&[Modifier::Without(FilterKind::AskHn)])
    }

    pub fn show_hn() -> Self {
        Self::new(&[Modifier::With(FilterKind::ShowHn)])
    }

    pub fn no_show_hn() -> Self {
        Self::new(&[Modifier::Without(FilterKind::ShowHn)])
    }

    pub fn bignews() -> Self {
        Self::new(&[Modifier::With(FilterKind::LargeNewspaper)])
    }

    pub fn no_bignews() -> Self {
        Self::new(&[Modifier::Without(FilterKind::LargeNewspaper)])
    }

    pub fn bigtech() -> Self {
        Self::new(&[Modifier::With(FilterKind::BigTech)])
    }

    pub fn no_bigtech() -> Self {
        Self::new(&[Modifier::Without(FilterKind::BigTech)])
    }

    pub fn no_bignews_no_bigtech() -> Self {
        Self::new(&[
            Modifier::Without(FilterKind::LargeNewspaper),
            Modifier::Without(FilterKind::BigTech),
        ])
    }

    pub fn ask_show_hn() -> Self {
        Self::new(&[
            Modifier::With(FilterKind::AskHn),
            Modifier::With(FilterKind::ShowHn),
        ])
    }

    /// Returns the name of the page. This should be used for the S3 object.
    /// Users will access the page at `https://suckless.hn/${name}`.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn stories(&self) -> &[Rc<StoryWithFilters>] {
        &self.stories
    }

    /// If the story was flagged by filters this page is happy with, push it to
    /// the list of stories we render for this page.
    pub fn push(&mut self, story: Rc<StoryWithFilters>) {
        // all "-" modifiers are conjunctive
        let removed_from_page = || {
            self.modifiers
                .iter()
                .filter_map(|modifier| match modifier {
                    Modifier::Without(filter) => Some(filter),
                    Modifier::With(_) => None,
                })
                .any(|filter| story.filters.contains(filter))
        };

        // all "+" modifiers are disjunctive
        let included_in_page = || {
            let mut plus_modifiers = self
                .modifiers
                .iter()
                .filter_map(|modifier| match modifier {
                    Modifier::With(filter) => Some(filter),
                    Modifier::Without(_) => None,
                })
                .peekable();

            if plus_modifiers.peek().is_some() {
                // includes only stories which have at least one of the "+"
                // modifier filter
                plus_modifiers.any(|filter| story.filters.contains(filter))
            } else {
                // if there's no "+" modifier, include all stories
                true
            }
        };

        if !removed_from_page() && included_in_page() {
            self.stories.push(story);
        }
    }

    pub fn len(&self) -> usize {
        self.stories.len()
    }

    /// Compiles and uploads pages for both themes.
    pub async fn upload(
        self,
        conf: &conf::Conf,
        html_engine: &Template,
    ) -> Result<()> {
        let jobs = vec![
            self.upload_theme(conf, html_engine, Theme::Dark),
            self.upload_theme(conf, html_engine, Theme::Light),
        ];

        for job in futures::future::join_all(jobs).await {
            job?;
        }

        Ok(())
    }

    /// Compiles the html and uploads the page to S3 bucket.
    async fn upload_theme(
        &self,
        conf: &conf::Conf,
        html_engine: &Template,
        theme: Theme,
    ) -> Result<()> {
        let html = html_engine.render(&self, theme)?;

        if conf.store_html_locally {
            let file_path = format!("pages/{}/{}.html", theme, self.name());

            log::trace!("Storing page {} ({})...", file_path, theme);
            fs::write(file_path, html.as_bytes()).await?;
            Ok(())
        } else {
            let object_path = theme.object_path(self.name());

            log::trace!("Uploading page {} ({})...", object_path, theme);
            let (_, code) = conf
                .bucket
                .put_object_with_content_type(
                    object_path.as_ref(),
                    html.as_bytes(),
                    "text/html",
                )
                .await?;

            if code != 200 {
                log::error!(
                    "Cannot upload page {} (code {})",
                    self.name(),
                    code
                );
                // hack to return error
                Err(Box::new(io::Error::from_raw_os_error(1)))
            } else {
                Ok(())
            }
        }
    }

    fn new(modifiers: &'static [Modifier]) -> Self {
        let name = {
            // we could introduce a new filter which would just be 1 for every
            // story, but that seems like to much work
            // this is kind of hacky but will do for now
            if modifiers.is_empty() {
                "+all".to_string()
            } else {
                let mut modifiers = modifiers.to_vec();
                modifiers.sort();
                modifiers.iter().fold(String::new(), |mut acc, modifier| {
                    acc.push_str(&modifier.to_string());
                    acc
                })
            }
        };

        Self {
            name,
            modifiers,
            stories: Vec::new(),
        }
    }
}

/// Creates list of pages for suckless.hn and populates them with stories from
/// the database.
pub fn populate(
    conn: &Connection,
    top_stories: Vec<StoryId>,
    page_limit: usize,
) -> Vec<Page> {
    let mut pages = list();
    for top_story_id in top_stories {
        if let Ok(Some(story)) = db::select_story(&conn, top_story_id) {
            let story = Rc::new(story);
            for page in &mut pages {
                if page.len() < page_limit {
                    page.push(Rc::clone(&story));
                }
            }

            let all_pages_full =
                pages.iter().all(|page| page.len() > page_limit);
            if all_pages_full {
                break;
            }
        }
    }

    pages
}

// List of all suckless.hn pages.
fn list() -> Vec<Page> {
    vec![
        Page::all(),
        Page::ask_hn(),
        Page::ask_show_hn(),
        Page::bignews(),
        Page::bigtech(),
        Page::no_ask_hn(),
        Page::no_bignews_no_bigtech(),
        Page::no_bignews(),
        Page::no_bigtech(),
        Page::no_show_hn(),
        Page::show_hn(),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use {super::*, FilterKind::*};

    #[test]
    fn it_pushes_story() {
        let empty_story = Rc::new(StoryWithFilters::random(vec![]));
        let ask_hn_story = Rc::new(StoryWithFilters::random(vec![AskHn]));
        let show_hn_story = Rc::new(StoryWithFilters::random(vec![ShowHn]));
        let amfg_bignews_story =
            Rc::new(StoryWithFilters::random(vec![LargeNewspaper, BigTech]));

        let mut ask_hn_page = Page::ask_hn();
        ask_hn_page.push(Rc::clone(&empty_story));
        assert!(ask_hn_page.stories.is_empty());
        ask_hn_page.push(Rc::clone(&show_hn_story));
        assert!(ask_hn_page.stories.is_empty());
        ask_hn_page.push(Rc::clone(&ask_hn_story));
        assert!(!ask_hn_page.stories.is_empty());
        assert_eq!("+askhn", ask_hn_page.name());

        let mut show_hn_page = Page::show_hn();
        show_hn_page.push(Rc::clone(&empty_story));
        assert!(show_hn_page.stories.is_empty());
        show_hn_page.push(Rc::clone(&amfg_bignews_story));
        assert!(show_hn_page.stories.is_empty());
        show_hn_page.push(Rc::clone(&show_hn_story));
        assert!(!show_hn_page.stories.is_empty());
        assert_eq!("+showhn", show_hn_page.name());

        let mut default_page = Page::no_bignews_no_bigtech();
        default_page.push(Rc::clone(&amfg_bignews_story));
        assert!(default_page.stories.is_empty());
        default_page.push(Rc::clone(&empty_story));
        assert!(!default_page.stories.is_empty());
        default_page.push(Rc::clone(&show_hn_story));
        assert_eq!(2, default_page.stories.len());
        assert_eq!("-amfg-bignews", default_page.name());

        let mut ask_show_hn_page = Page::ask_show_hn();
        ask_show_hn_page.push(Rc::clone(&empty_story));
        assert!(ask_show_hn_page.stories.is_empty());
        ask_show_hn_page.push(Rc::clone(&amfg_bignews_story));
        assert!(ask_show_hn_page.stories.is_empty());
        ask_show_hn_page.push(Rc::clone(&show_hn_story));
        assert!(!ask_show_hn_page.stories.is_empty());
        ask_show_hn_page.push(Rc::clone(&ask_hn_story));
        assert_eq!(2, ask_show_hn_page.stories.len());
        assert_eq!("+askhn+showhn", ask_show_hn_page.name());
    }

    #[test]
    fn it_populates_pages() -> Result<()> {
        let limit_stories_per_page = 7;
        let conn = db::tests::test_conn()?;

        let stories = &[
            (Story::random_url(), vec![AskHn, ShowHn]),
            (Story::random_url(), vec![LargeNewspaper]),
            (Story::random_url(), vec![]),
            (Story::random_url(), vec![]),
            (Story::random_url(), vec![]),
            (Story::random_url(), vec![BigTech, LargeNewspaper]),
            (Story::random_url(), vec![BigTech]),
            (Story::random_url(), vec![AskHn]),
        ];
        let ids = stories.iter().map(|(story, _)| story.id).collect();
        db::tests::insert_test_data(&conn, stories)?;

        let pages = populate(&conn, ids, limit_stories_per_page);

        let pages: HashMap<_, _> = pages
            .into_iter()
            .map(|page| (page.name().to_string(), page))
            .collect();

        println!("Pages: {:#?}", pages);
        assert_eq!(2, pages.get("+askhn").unwrap().len());
        assert_eq!(6, pages.get("-askhn").unwrap().len());
        assert_eq!(5, pages.get("-amfg-bignews").unwrap().len());
        assert_eq!(6, pages.get("-amfg").unwrap().len());
        assert_eq!(1, pages.get("+askhn+showhn").unwrap().len());
        assert_eq!(limit_stories_per_page, pages.get("+all").unwrap().len());

        Ok(())
    }
}
