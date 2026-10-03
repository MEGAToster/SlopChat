use serde_json::{json, Value};

pub(super) fn stop_hint(tool: &str) -> String {
    format!(
        "{tool} is used up for this turn — you may still call other tools, or answer now using the results above"
    )
}
pub(super) const LEAK_NOTE: &str = "_(the model tried to call tools in an unsupported format)_";
pub(super) const LENGTH_NOTE: &str = "_(the answer hit the token limit and may be cut off)_";
pub(super) const LAST_ROUND_NOTICE: &str =
    "One research step left. Use it only if essential; after it you must answer.";

pub(super) fn wrap_up_message() -> Value {
    json!({
        "role": "user",
        "content": "Stop calling tools. Answer now using the results above.",
    })
}
