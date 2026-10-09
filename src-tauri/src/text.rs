// Truncate by characters, not bytes, so multi-byte text can't be split mid-codepoint.
pub fn truncate(text: &str, cap: usize) -> String {
    text.chars().take(cap).collect()
}
