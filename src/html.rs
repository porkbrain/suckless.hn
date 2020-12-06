//! Based on preset filter groups we generate html page from
//! [handlebars][handlebars] template.
//!
//! [handlebars]: https://handlebarsjs.com/guide/

use {handlebars::Handlebars, serde_json::json};

use crate::{filter::Page, prelude::*};

// The template handlebars file we use to create each html page.
const TEMPLATE_CONTENTS: &str =
    include_str!("assets/front-page.handlebars.html");

// Handlebars provides interface for multiple templates, but we don't need this.
const TEMPLATE_NAME: &str = "front-page";

pub struct Template(Handlebars<'static>);

impl Template {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string(TEMPLATE_NAME, TEMPLATE_CONTENTS)?;

        Ok(Self(handlebars))
    }

    /// Given page populated with stories, we render it against the handlebars
    /// template.
    pub fn render(&self, page: &Page) -> Result<String> {
        let json = json!({
            "stories": page.stories(),
        });

        let html = self.0.render(TEMPLATE_NAME, &json)?;
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{db, filter::page},
    };

    #[test]
    fn it_should_render_page() -> Result<()> {
        let engine = Template::new()?;
        let conn = db::tests::test_conn()?;

        let story = Story::random_url();
        let title = story.title.clone();
        let stories = &[(story, vec![FilterKind::AskHn])];
        let ids = stories.iter().map(|(story, _)| story.id).collect();
        db::tests::insert_test_data(&conn, stories)?;

        let pages = page::populate(&conn, ids, 1);
        let ask_hn_page =
            pages.into_iter().find(|p| p.name() == "+askhn").unwrap();

        let html = engine.render(ask_hn_page)?;
        println!("{}", html);
        assert!(html.contains(&title));

        Ok(())
    }
}
