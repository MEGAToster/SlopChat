use serde_json::{json, Value};

use super::hit::{Batch, Hit};
use super::{BackendCtx, SearchError, TIMEOUT};

pub async fn search(query: &str, n: u32, ctx: &BackendCtx<'_>) -> Result<Batch, SearchError> {
    let body = json!({
        "query": query,
        "max_results": n,
        "search_depth": "basic",
        "topic": "general",
    });

    let value = super::send_json(
        crate::providers::client()
            .post("https://api.tavily.com/search")
            .header("Authorization", format!("Bearer {}", ctx.key))
            .timeout(TIMEOUT)
            .json(&body),
    )
    .await?;

    Ok(parse(&value))
}

fn parse(value: &Value) -> Batch {
    let hits = value
        .get("results")
        .and_then(Value::as_array)
        .map(|results| {
            results
                .iter()
                .map(|item| Hit {
                    title: text(item, "title"),
                    url: text(item, "url"),
                    snippet: text(item, "content"),
                    published: item
                        .get("published_date")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default();

    Batch {
        hits,
        units: 1,
        cost_usd: 0.0,
    }
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_results() {
        let value: Value = serde_json::from_str(
            r#"{"results":[
                {"title":"One","url":"https://a.test","content":"first","published_date":"2024-01-02"},
                {"title":"Two","url":"https://b.test","content":"second"}
            ]}"#,
        )
        .unwrap();

        let batch = parse(&value);
        assert_eq!(batch.units, 1);
        assert_eq!(batch.hits[0].title, "One");
        assert_eq!(batch.hits[0].snippet, "first");
        assert_eq!(batch.hits[0].published.as_deref(), Some("2024-01-02"));
        assert_eq!(batch.hits[1].url, "https://b.test");
    }
}
