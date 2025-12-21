use super::HtmlFilter;
use scraper::ElementRef;

pub struct FilterHtmlAttributes {
    attributes: Vec<String>,
}

impl FilterHtmlAttributes {
    pub fn new<S: Into<String>>(attributes: Vec<S>) -> Self {
        Self {
            attributes: attributes.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl Default for FilterHtmlAttributes {
    fn default() -> Self {
        Self {
            attributes: vec!["id".into(), "class".into()],
        }
    }
}

impl HtmlFilter for FilterHtmlAttributes {
    fn strip_element(&self, element: &ElementRef<'_>) -> String {
        let tag_name = element.value().name();
        let mut attributes = String::new();

        for (name, value) in element.value().attrs() {
            if self.attributes.iter().any(|a| a == name) {
                attributes.push_str(&format!(" {name}=\"{value}\""));
            }
        }

        let void_element = if element.has_children() { "" } else { " /" };
        format!("<{}{}{}>", tag_name, attributes, void_element)
    }
}
