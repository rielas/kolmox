use super::common;

pub fn get_content(page: &str) -> Option<String> {
    common::get_content_with(page, &["article", "main", "body"], "span")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::read_test_file as read_from_file;

    #[test]
    fn test_extract_content_from_grok_page() {
        let page_html =
            read_from_file("../../../dataset/grokvswiki/page/Jordan_Smith_(musician).html");
        let extracted = get_content(&page_html).unwrap();
        assert!(!extracted.contains('<'));
        assert!(!extracted.trim().is_empty());
        println!("{extracted}");
    }

    #[test]
    fn test_grok_punctuation_normalization() {
        let page_html =
            read_from_file("../../../dataset/grokvswiki/page/Jordan_Smith_(musician).html");
        let extracted = get_content(&page_html).unwrap();
        assert!(!extracted.contains(" ,"));
        assert!(!extracted.contains(" ."));
        assert!(
            extracted.contains("\"Great Is Thy Faithfulness\"")
                || extracted.contains("\"Great Is Thy Faithfulness,\"")
        );
    }
}
