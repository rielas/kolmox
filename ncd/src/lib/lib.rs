pub mod compress;
pub mod filter;

use crate::compress::Compressor;
use filter::HtmlFilter;

pub fn brotli_default_filter_attributes(page_a: &str, page_b: &str) -> f64 {
    let stripper = filter::filter_attributes::FilterAttributes::default();
    let stripped_a = stripper.process_document(page_a);
    let stripped_b = stripper.process_document(page_b);
    let compressor = compress::brotli::CompressBrotli::recommended();
    compressor.get_distance(&stripped_a, &stripped_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_simple_pages() {
        let page_a = r#"<html>
    <head>
        <title>This title</title>
    </head>
    <body>
        <p class="hello">Hello, world!</p>
    </body>
</html>"#;
        let page_b = r#"<html>
    <head>
        <title>A Different Test</title>
    </head>
    <body>
        <p class="hello">Good bye, world!</p>
    </body>
</html>"#;
        assert_approx_eq!(brotli_default_filter_attributes(page_a, page_b), 0.0, 0.1);
    }
}
