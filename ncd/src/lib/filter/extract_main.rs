use scraper::{ElementRef, Html, Selector};

pub fn extract_main(html: String) -> String {
    let document = Html::parse_document(&html);
    let selector = Selector::parse("main").unwrap();

    if let Some(main_elem) = document.select(&selector).next() {
        let mut result = String::new();
        traverse_element(&main_elem, &mut result);
        result
    } else {
        String::new()
    }
}

fn strip_element(element: &ElementRef<'_>) -> String {
    let tag_name = element.value().name();
    let attrs = element
        .value()
        .attrs()
        .map(|(k, v)| format!(" {}=\"{}\"", k, v))
        .collect::<String>();

    if element.has_children() {
        format!("<{}{}>", tag_name, attrs)
    } else {
        format!("<{}{} />", tag_name, attrs)
    }
}

fn traverse_element(element: &ElementRef<'_>, result: &mut String) {
    let begin = strip_element(element);
    result.push_str(&begin);

    if element.has_children() {
        for child in element.child_elements() {
            traverse_element(&child, result);
        }

        let tag_name = element.value().name();
        result.push_str(&format!("</{}>", tag_name));
    }
}
