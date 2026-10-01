use serde_json::Value;

pub fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[derive(Debug)]
pub struct Hit {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published: Option<String>,
}

#[derive(Debug, Default)]
pub struct Batch {
    pub hits: Vec<Hit>,
    pub units: u32,
    pub cost_usd: f64,
}
