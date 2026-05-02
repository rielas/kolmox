use scraper::{Html, Selector};

pub const EXCLUDED_SELECTORS: &[&str] = &[
    "div#references",
    "table",
    ".infobox",
    "nav",
    ".toc",
    "footer",
    "aside",
];

pub fn get_content_with(
    page: &str,
    container_selectors: &[&str],
    candidate_selector: &str,
) -> Option<String> {
    let document = Html::parse_document(page);

    let container = container_selectors.iter().find_map(|s| {
        Selector::parse(s)
            .ok()
            .and_then(|sel| document.select(&sel).next())
    })?;

    let candidate_sel = Selector::parse(candidate_selector).ok()?;

    let mut excluded = Vec::new();
    for ex in EXCLUDED_SELECTORS {
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

pub fn clean_text(raw: &str) -> String {
    let mut s = raw.replace('\n', " ");
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
    out = out.replace(" \"", "\"");

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
    out = out.replace(" 's", "'s");
    out = out.replace(" 're", "'re");
    out = out.replace(" n't", "n't");
    out = out.replace(" 've", "'ve");
    out = out.replace(" 'll", "'ll");
    out = out.replace(" 'd", "'d");
    out = out.replace(" 'm", "'m");

    out
}
