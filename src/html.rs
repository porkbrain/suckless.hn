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
    ///
    /// Dark allows us to compile for dark theme.
    pub fn render(&self, page: &Page, theme: Theme) -> Result<String> {
        let dark = matches!(theme, Theme::Dark);
        let json = json!({
            "name": page.name(),
            "stories": page.stories(),
            "dark": dark
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

        let mut story1 = Story::random_url();
        story1.kind = StoryKind::Url("https://porkbrain.com".to_string());
        story1.title = "Every Model Learned by Gradient Descent".to_string();
        let mut story2 = Story::random_url();
        story2.title = "LinkedIn’s Alternate Universe".to_string();
        story2.archive_url = Some("https://example.com".to_string());

        let stories = &[(story1.clone(), vec![]), (story2.clone(), vec![])];
        let ids = stories.iter().map(|(story, _)| story.id).collect();
        db::tests::insert_test_data(&conn, stories)?;

        let pages = page::populate(&conn, ids, 5);
        let ask_hn_page =
            pages.into_iter().find(|p| p.name() == "+all").unwrap();

        let dark_html = engine.render(&ask_hn_page, Theme::Dark)?;

        assert!(dark_html.contains(&story1.title));
        assert!(dark_html.contains(&story2.title));
        assert!(dark_html.contains(&story2.archive_url.unwrap()));

        assert!(dark_html.contains("dark.css"));
        assert!(!dark_html.contains("light.css"));

        let light_html = engine.render(&ask_hn_page, Theme::Light)?;
        assert!(light_html.contains("light.css"));
        assert!(!light_html.contains("dark.css"));

        Ok(())
    }
}
