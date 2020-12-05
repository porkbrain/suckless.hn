//! Based on preset filter groups we generate html page from
//! [handlebars][handlebars] template.
//!
//! [handlebars]: https://handlebarsjs.com/guide/

use handlebars::Handlebars;

use crate::prelude::*;

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

    // TODO: pub fn render(&self, stories: &[Story])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_render_page() -> Result<()> {
        let engine = Template::new()?;

        // TODO: render

        Ok(())
    }
}
