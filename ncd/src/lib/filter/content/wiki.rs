use scraper::{ElementRef, Html, Selector};

fn choose_container(document: &Html) -> Option<ElementRef<'_>> {
    let try_sel = |s: &str| {
        Selector::parse(s)
            .ok()
            .and_then(|sel| document.select(&sel).next())
    };
    try_sel("div#bodyContent")
        .or_else(|| try_sel("article"))
        .or_else(|| try_sel("main"))
        .or_else(|| try_sel("body"))
}

fn clean_text(raw: &str) -> String {
    let mut s = raw.replace('\n', " ");
    // remove simple bracketed citations like [1], [a]
    loop {
        if let Some(a) = s.find('[') {
            if let Some(b) = s[a..].find(']') {
                s.replace_range(a..a + b + 1, " ");
                continue;
            }
        }
        break;
    }
    let mut out = s.split_whitespace().collect::<Vec<_>>().join(" ");
    // tidy punctuation spacing
    for (pat, rep) in [
        (" ,", ","),
        (" .", "."),
        (" :", ":"),
        (" ;", ";"),
        (" !", "!"),
        (" ?", "?"),
        (" )", ")"),
    ] {
        out = out.replace(pat, rep);
    }
    out = out.replace("\" ", "\"");
    out = out.replace("' ", "'");
    // remove space before closing quote
    out = out.replace(" \"", "\"");

    // normalize quotes: remove space after opening quote and ensure space after closing quote
    fn normalize_quotes(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0usize;
        let mut in_double = false;
        let mut in_single = false;
        while i < chars.len() {
            let c = chars[i];
            if c == '"' {
                out.push(c);
                if !in_double {
                    in_double = true;
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                } else {
                    in_double = false;
                    if i + 1 < chars.len() {
                        let nx = chars[i + 1];
                        if !nx.is_whitespace() && ![',', '.', ':', ';', '!', '?', ')'].contains(&nx)
                        {
                            out.push(' ');
                        }
                    }
                    i += 1;
                    continue;
                }
            } else if c == '\'' {
                out.push(c);
                if !in_single {
                    in_single = true;
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                } else {
                    in_single = false;
                    if i + 1 < chars.len() {
                        let nx = chars[i + 1];
                        if !nx.is_whitespace() && ![',', '.', ':', ';', '!', '?', ')'].contains(&nx)
                        {
                            out.push(' ');
                        }
                    }
                    i += 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        out
    }

    out = normalize_quotes(&out);

    // final pass: normalize spacing around quotes and parentheses more robustly
    fn normalize_spacing(s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut out = String::with_capacity(s.len());
        let mut i = 0usize;
        let mut in_double = false;
        let mut in_single = false;

        while i < chars.len() {
            let c = chars[i];

            if c == '(' || c == '[' || c == '{' {
                out.push(c);
                i += 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                continue;
            }

            if c.is_whitespace() {
                if i + 1 < chars.len() && (chars[i + 1] == '"' || chars[i + 1] == '\'') {
                    if let Some(last) = out.chars().rev().find(|ch| !ch.is_whitespace()) {
                        if ['(', '[', '{'].contains(&last) {
                            i += 1;
                            continue;
                        } else {
                            if out
                                .chars()
                                .last()
                                .map(|c| !c.is_whitespace())
                                .unwrap_or(false)
                            {
                                out.push(' ');
                            }
                            i += 1;
                            continue;
                        }
                    } else {
                        i += 1;
                        continue;
                    }
                }

                if out
                    .chars()
                    .last()
                    .map(|c| c.is_whitespace())
                    .unwrap_or(false)
                {
                    i += 1;
                    continue;
                } else {
                    out.push(' ');
                    i += 1;
                    continue;
                }
            }

            if c == '"' {
                if !in_double {
                    if out
                        .chars()
                        .last()
                        .map(|ch| !ch.is_whitespace() && !['(', '[', '{'].contains(&ch))
                        .unwrap_or(false)
                    {
                        out.push(' ');
                    }
                    out.push('"');
                    in_double = true;
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                } else {
                    out.push('"');
                    in_double = false;
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < chars.len() {
                        let nx = chars[i];
                        if !nx.is_whitespace()
                            && ![',', '.', ':', ';', '!', '?', ')', ']', '}'].contains(&nx)
                        {
                            out.push(' ');
                        }
                    }
                    continue;
                }
            }

            if c == '\'' {
                // if this looks like an apostrophe/contraction (next is alnum), treat specially
                if i + 1 < chars.len() && chars[i + 1].is_alphanumeric() {
                    out.push('\'');
                    i += 1;
                    continue;
                }

                if !in_single {
                    if out
                        .chars()
                        .last()
                        .map(|ch| !ch.is_whitespace() && !['(', '[', '{'].contains(&ch))
                        .unwrap_or(false)
                    {
                        out.push(' ');
                    }
                    out.push('\'');
                    in_single = true;
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                } else {
                    out.push('\'');
                    in_single = false;
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < chars.len() {
                        let nx = chars[i];
                        if !nx.is_whitespace()
                            && ![',', '.', ':', ';', '!', '?', ')', ']', '}'].contains(&nx)
                        {
                            out.push(' ');
                        }
                    }
                    continue;
                }
            }

            out.push(c);
            i += 1;
        }

        out
    }

    out = normalize_spacing(&out);
    // Fix common contraction/possessive spacing that may remain
    out = out.replace(" 's", "'s");
    out = out.replace(" 're", "'re");
    out = out.replace(" n't", "n't");
    out = out.replace(" 've", "'ve");
    out = out.replace(" 'll", "'ll");
    out = out.replace(" 'd", "'d");
    out = out.replace(" 'm", "'m");

    out
}

pub fn get_content(page: &str) -> Option<String> {
    let document = Html::parse_document(page);
    let container = choose_container(&document)?;

    let candidate_sel = Selector::parse("p").ok()?;
    let excluded_selectors = [
        "div#references",
        "table",
        ".infobox",
        "nav",
        ".toc",
        "footer",
        "aside",
    ];

    // collect excluded candidate nodes (descendants of excluded containers)
    let mut excluded = Vec::new();
    for ex in &excluded_selectors {
        if let Ok(ex_sel) = Selector::parse(ex) {
            for ex_container in container.select(&ex_sel) {
                for c in ex_container.select(&candidate_sel) {
                    excluded.push(c);
                }
            }
        }
    }

    let mut paragraphs = Vec::new();

    for node in container.select(&candidate_sel) {
        if excluded.iter().any(|e| e == &node) {
            continue;
        }

        let raw = node.text().collect::<Vec<_>>().join(" ");
        let text = clean_text(&raw);
        if text.split_whitespace().count() < 8 {
            continue;
        }
        paragraphs.push(text);
    }

    if paragraphs.is_empty() {
        None
    } else {
        Some(paragraphs.join("\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::read_test_file as read_from_file;

    #[test]
    fn test_extract_content_from_wiki_page() {
        let page_html =
            read_from_file("../../../dataset/grokvswiki/wiki/Jordan_Smith_(musician).html");
        let extracted = get_content(&page_html).expect("expected content");
        // ensure there are no raw HTML tags
        assert!(!extracted.contains('<'));
        assert!(!extracted.trim().is_empty());
        println!("{extracted}");
    }

    #[test]
    fn test_wiki_punctuation_normalization() {
        let page_html =
            read_from_file("../../../dataset/grokvswiki/wiki/Jordan_Smith_(musician).html");
        let extracted = get_content(&page_html).expect("expected content");
        // no space before comma/period
        assert!(!extracted.contains(" ,"));
        assert!(!extracted.contains(" ."));
        // quoted song titles should appear (spacing validated visually)
        assert!(extracted.contains("\"Great Is Thy Faithfulness\""));
        assert!(extracted.contains("\"Somebody to Love\""));
    }
}
