use super::Processor;
use scraper::{Html, Selector};

pub struct ExtractMain {}

impl Processor for ExtractMain {
    fn process_document(&self, page: &str) -> String {
        let document = Html::parse_document(&page);
        let selector = Selector::parse("main").unwrap();

        if let Some(main_elem) = document.select(&selector).next() {
            let inner_html = main_elem.inner_html();
            inner_html.trim().to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_main() {
        let page = r#"<html>
    <head>
        <title>Test</title>
    </head>
    <body>
        <header>This is the header</header>
        <main>
            <p>This is the main content.</p>
        </main>
        <footer>This is the footer</footer>
    </body>
</html>"#;
        let extractor = ExtractMain {};
        let extracted = extractor.process_document(page);
        assert_eq!(extracted, r#"<p>This is the main content.</p>"#);
    }
}
