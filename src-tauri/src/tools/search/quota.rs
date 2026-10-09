use serde_json::Value;

use super::SearchError;

pub(super) fn classify(status: u16, body: &str) -> SearchError {
    if matches!(status, 402 | 429 | 432 | 433) || explicit_quota(body) {
        SearchError::Exhausted(reason(body).unwrap_or_else(|| format!("HTTP {status}")))
    } else {
        SearchError::Failed(format!("HTTP {status}"))
    }
}

// Only unambiguous billing phrases; a bare "limit" or "plan" in an error body
// must not lock a provider out for the rest of the month.
fn explicit_quota(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    [
        "out of credits",
        "insufficient credits",
        "not enough credits",
        "quota exceeded",
        "quota has been exceeded",
        "credit limit reached",
        "monthly quota",
        "free allowance",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(super) fn reason(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    for pointer in ["/error", "/message", "/detail", "/error/message"] {
        if let Some(text) = value.pointer(pointer).and_then(Value::as_str) {
            return Some(text.to_string());
        }
    }
    None
}
