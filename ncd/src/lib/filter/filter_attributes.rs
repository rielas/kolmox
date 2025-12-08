use super::HtmlFilter;
use scraper::ElementRef;

pub struct FilterAttributes {}

impl HtmlFilter for FilterAttributes {
    fn strip_element(&self, element: &ElementRef<'_>) -> String {
        let tag_name = element.value().name();
        let mut attributes = String::new();

        for (name, value) in element.value().attrs() {
            if ["id", "class"].contains(&name) {
                attributes.push_str(&format!(" {name}=\"{value}\""));
            }
        }

        let void_element = if element.has_children() { "" } else { " /" };
        format!("<{}{}{}>", tag_name, attributes, void_element)
    }
}
