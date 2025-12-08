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
