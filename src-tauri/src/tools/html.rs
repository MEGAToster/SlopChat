const SKIP: &[&str] = &["script", "style", "nav", "footer", "svg", "head", "noscript"];
const BLOCK: &[&str] = &[
    "p", "div", "br", "li", "ul", "ol", "h1", "h2", "h3", "h4", "h5", "h6", "tr", "section",
    "article", "header", "table", "pre", "blockquote", "hr",
];

pub fn to_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut skip = 0u32;
    let mut index = 0;

    while index < input.len() {
        let character = input[index..].chars().next().unwrap();

        if character == '<' {
            if let Some(offset) = input[index..].find('>') {
                let raw = &input[index + 1..index + offset];
                let closing = raw.starts_with('/');
                let name = tag_name(raw);

                if SKIP.contains(&name.as_str()) {
                    if closing {
                        skip = skip.saturating_sub(1);
                    } else {
                        skip += 1;
                    }
                } else if skip == 0 && BLOCK.contains(&name.as_str()) {
                    out.push('\n');
                }

                index += offset + 1;
                continue;
            }
        }

        if skip == 0 {
            out.push(character);
        }
        index += character.len_utf8();
    }

    collapse(&decode_entities(&out))
}

fn tag_name(raw: &str) -> String {
    raw.trim_start_matches('/')
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
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

fn collapse(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags_scripts_and_collapses() {
        let html = "<html><head><style>body{}</style></head><body><h1>Hi</h1><p>a &amp; b</p><script>evil()</script><div>bye</div></body></html>";
        let text = to_text(html);
        assert_eq!(text, "Hi a & b bye");
        assert!(!text.contains("evil"));
    }
}
