use serde_json::{json, Value};

use super::hit::{text, Batch, Hit};
use super::{BackendCtx, SearchError, TIMEOUT};
use crate::config::ExaSettings;

pub async fn search(query: &str, n: u32, ctx: &BackendCtx<'_>) -> Result<Batch, SearchError> {
    let body = json!({
        "query": query,
        "type": ctx.settings.exa.search_type,
        "numResults": n,
        "contents": contents(&ctx.settings.exa),
    });

    let value = super::send_json(
        crate::providers::client()
            .post("https://api.exa.ai/search")
            .header("x-api-key", ctx.key)
            .timeout(TIMEOUT)
            .json(&body),
    )
    .await?;

    let cost_usd = value
        .get("costDollars")
        .and_then(|cost| cost.get("total"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);

    let hits = value
        .get("results")
        .and_then(Value::as_array)
        .map(|results| results.iter().map(hit).collect())
        .unwrap_or_default();

    Ok(Batch {
        hits,
        units: 1,
        cost_usd,
    })
}

fn hit(result: &Value) -> Hit {
    Hit {
        title: text(result, "title"),
        url: text(result, "url"),
        snippet: snippet(result),
        published: result
            .get("publishedDate")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn snippet(result: &Value) -> String {
    if let Some(highlights) = result.get("highlights").and_then(Value::as_array) {
        return highlights
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" ");
    }
    text(result, "text")
}

fn contents(exa: &ExaSettings) -> Value {
    match exa.contents.as_str() {
        "highlights" => json!({ "highlights": { "maxCharacters": exa.max_characters } }),
        "text" => json!({ "text": { "maxCharacters": exa.max_characters } }),
        _ => json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_results_with_highlight_snippets() {
        let value: Value = serde_json::from_str(
            r#"{"costDollars":{"total":0.007},"results":[
                {"title":"One","url":"https://a.test","highlights":["alpha","beta"]},
                {"title":"Two","url":"https://b.test","text":"gamma","publishedDate":"2024-01-02"}
            ]}"#,
        )
        .unwrap();

        let hits: Vec<Hit> = value["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(hit)
            .collect();

        assert_eq!(hits[0].title, "One");
        assert_eq!(hits[0].snippet, "alpha beta");
        assert_eq!(hits[1].snippet, "gamma");
        assert_eq!(hits[1].published.as_deref(), Some("2024-01-02"));
    }
}
