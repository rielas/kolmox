pub mod filter_attributes;
pub mod strip_content;

use scraper::{ElementRef, Html};

pub trait HtmlFilter {
    fn process_document(&self, page: &str) -> String {
        let document = Html::parse_document(page);
        let mut result = String::new();
        self.traverse_element(&document.root_element(), &mut result);
        result
    }

    fn strip_element(&self, element: &ElementRef<'_>) -> String;

    fn traverse_element(&self, element: &ElementRef<'_>, result: &mut String) {
        let begin = self.strip_element(element);

        result.push_str(&begin);

        if element.has_children() {
            for child in element.child_elements() {
                self.traverse_element(&child, result);
            }

            let tag_name = element.value().name();
            result.push_str(&format!("</{}>", tag_name));
        }
    }
}

pub struct FilterPipeline {
    filters: Vec<Box<dyn Fn(String) -> String>>,
}

impl FilterPipeline {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn then<F: HtmlFilter + 'static>(mut self, filter: F) -> Self {
        self.filters
            .push(Box::new(move |html| filter.process_document(&html)));
        self
    }

    pub fn apply(&self, html: &str) -> String {
        self.filters.iter().fold(html.to_string(), |acc, f| f(acc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_pipeline() {
        let page = r#"<html>
    <head>
        <title>Test</title>
    </head>
    <body>
        <p class="hello">Hello, world!</p>
    </body>
</html>"#;

        let pipeline = FilterPipeline::new()
            .then(strip_content::StripContent {})
            .then(filter_attributes::FilterAttributes::default());

        let result = pipeline.apply(page);

        assert_eq!(
            result,
            r#"<html><head><title /></head><body><p class="hello" /></body></html>"#
        );
    }
}
