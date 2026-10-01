const SKIP: &[&str] = &[
    "script", "style", "nav", "footer", "svg", "head", "noscript", "aside", "form", "button",
];
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];
const BLOCK: &[&str] = &[
    "p", "div", "br", "li", "ul", "ol", "h1", "h2", "h3", "h4", "h5", "h6", "section", "article",
    "header", "table", "pre", "blockquote", "hr",
];

pub fn to_text(input: &str) -> String {
    collapse(&decode_entities(&scan(content_region(input))))
}

// Single-line variant for search snippets, where pipe rows and paragraphs are
// noise rather than structure.
pub fn to_line(input: &str) -> String {
    to_text(input).split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn title(input: &str) -> Option<String> {
    let raw = inner(input, "title")?;
    let text = decode_entities(&scan(raw));
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

// Documentation pages bury the article under nav/aside chrome; prefer the
// semantic container when one exists.
fn content_region(input: &str) -> &str {
    inner(input, "main")
        .or_else(|| inner(input, "article"))
        .unwrap_or(input)
}

fn inner<'a>(input: &'a str, tag: &str) -> Option<&'a str> {
    let lower = input.to_ascii_lowercase();
    let open = lower.find(&format!("<{tag}"))?;
    let start = open + lower[open..].find('>')? + 1;
    let end = start + lower[start..].find(&format!("</{tag}>"))?;
    Some(&input[start..end])
}

fn scan(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut skipped: Vec<String> = Vec::new();
    let mut cells = 0u32;
    let mut index = 0;

    while index < input.len() {
        let character = input[index..].chars().next().unwrap();

        if character == '<' {
            if let Some(offset) = input[index..].find('>') {
                let raw = &input[index + 1..index + offset];
                let closing = raw.starts_with('/');
                let name = tag_name(raw);
                let void = raw.trim_end().ends_with('/') || VOID.contains(&name.as_str());

                if closing {
                    if skipped.last().map(String::as_str) == Some(name.as_str()) {
                        skipped.pop();
                    }
                } else if SKIP.contains(&name.as_str()) || (!void && hidden(raw)) {
                    skipped.push(name);
                } else if skipped.is_empty() && name == "tr" {
                    out.push('\n');
                    cells = 0;
                } else if skipped.is_empty() && (name == "td" || name == "th") {
                    if cells > 0 {
                        out.push_str(" | ");
                    }
                    cells += 1;
                } else if skipped.is_empty() && BLOCK.contains(&name.as_str()) {
                    out.push('\n');
                }

                index += offset + 1;
                continue;
            }
        }

        if skipped.is_empty() {
            out.push(character);
        }
        index += character.len_utf8();
    }

    out
}

fn tag_name(raw: &str) -> String {
    raw.trim_start_matches('/')
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn hidden(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.contains("aria-hidden=\"true\"")
        || lower.contains("aria-hidden='true'")
        || lower.contains("aria-hidden=true")
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

// Collapses runs of spaces and blank lines while keeping paragraph breaks.
fn collapse(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut newlines = 0u32;
    let mut space = false;

    for character in input.chars() {
        if character == '\n' {
            newlines += 1;
            space = false;
        } else if character.is_whitespace() {
            space = true;
        } else {
            if newlines > 0 {
                out.push('\n');
                newlines = 0;
            } else if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            out.push(character);
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags_scripts_and_keeps_paragraphs() {
        let html = "<html><head><style>body{}</style></head><body><h1>Hi</h1><p>a &amp; b</p><script>evil()</script><div>bye</div></body></html>";
        let text = to_text(html);
        assert_eq!(text, "Hi\na & b\nbye");
        assert!(!text.contains("evil"));
    }

    #[test]
    fn prefers_main_over_surrounding_chrome() {
        let html = "<body><nav>menu</nav><main><p>Keep</p></main><aside>ad</aside></body>";
        assert_eq!(to_text(html), "Keep");
    }

    #[test]
    fn renders_tables_as_pipe_rows() {
        let html = "<table><tr><th>Model</th><th>Price</th></tr><tr><td>Opus</td><td>$5</td></tr></table>";
        assert_eq!(to_text(html), "Model | Price\nOpus | $5");
    }

    #[test]
    fn extracts_and_decodes_the_title() {
        let html = "<head><title>A &amp; B <span>docs</span></title></head><body>x</body>";
        assert_eq!(title(html).as_deref(), Some("A & B docs"));
    }

    #[test]
    fn skips_aria_hidden_subtrees() {
        let html = "<div>shown <span aria-hidden=\"true\">hidden <b>deep</b></span> tail</div>";
        assert_eq!(to_text(html), "shown tail");
    }

    #[test]
    fn to_line_flattens_newlines() {
        assert_eq!(to_line("<p>a</p><p>b</p>"), "a b");
    }
}
