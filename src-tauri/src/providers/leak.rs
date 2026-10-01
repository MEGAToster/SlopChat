// Models sometimes print their native tool-call syntax as plain text when the
// provider doesn't parse it. These markers let us hold it back and drop it.
const MARKERS: &[&str] = &[
    "<|DSML|",
    "<｜DSML｜",
    "<|tool▁calls▁begin|>",
    "<|tool▁call▁begin|>",
    "<|tool_calls_begin|>",
    "<|tool_call_begin|>",
    "<tool_call>",
    "[TOOL_CALLS]",
];

// Longest marker in bytes; streaming holds back this much to catch splits.
pub const MAX_MARKER: usize = 32;

pub fn find(text: &str) -> Option<usize> {
    MARKERS.iter().filter_map(|marker| text.find(marker)).min()
}

pub fn has(text: &str) -> bool {
    find(text).is_some()
}

// Keeps the prose before the first marker and drops the rest.
pub fn strip(text: &str) -> String {
    match find(text) {
        Some(index) => text[..index].trim_end().to_string(),
        None => text.trim_end().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_every_marker() {
        for marker in MARKERS {
            let text = format!("prose {marker} more");
            assert!(has(&text), "missed {marker}");
            assert_eq!(find(&text), Some(6), "wrong index for {marker}");
        }
    }

    #[test]
    fn detects_a_marker_split_across_deltas() {
        let mut buffered = String::new();
        for part in ["hello <|DS", "ML| calls>"] {
            buffered.push_str(part);
        }
        assert_eq!(find(&buffered), Some(6));
    }

    #[test]
    fn strips_markup_and_keeps_the_prose_before_it() {
        let text = "The answer is 42.\n<|DSML| calls> <|DSML| invoke name=\"web_fetch\">";
        assert_eq!(strip(text), "The answer is 42.");
    }

    #[test]
    fn leaves_clean_text_untouched() {
        assert_eq!(strip("plain text  "), "plain text");
    }
}
