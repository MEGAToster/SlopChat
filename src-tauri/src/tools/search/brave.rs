use serde_json::Value;

use super::hit::{Batch, Hit};
use super::{BackendCtx, SearchError, TIMEOUT};
use crate::tools::html;

const COST_PER_QUERY: f64 = 0.005;

pub async fn search(query: &str, n: u32, ctx: &BackendCtx<'_>) -> Result<Batch, SearchError> {
    let mut url =
        reqwest::Url::parse("https://api.search.brave.com/res/v1/web/search").expect("brave url");
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("count", &n.to_string());

    let value = super::send_json(
        crate::providers::client()
            .get(url)
            .header("X-Subscription-Token", ctx.key)
            .header("Accept", "application/json")
            .timeout(TIMEOUT),
    )
    .await?;

    Ok(parse(&value))
}

fn parse(value: &Value) -> Batch {
    let hits = value
        .pointer("/web/results")
        .and_then(Value::as_array)
        .map(|results| {
            results
                .iter()
                .map(|item| Hit {
                    title: text(item, "title"),
                    url: text(item, "url"),
                    snippet: html::to_text(&text(item, "description")),
                    published: item.get("age").and_then(Value::as_str).map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default();

    Batch {
        hits,
        units: 1,
        cost_usd: COST_PER_QUERY,
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
    fn strips_strong_tags_from_descriptions() {
        let value: Value = serde_json::from_str(
            r#"{"web":{"results":[
                {"title":"One","url":"https://a.test","description":"a <strong>bold</strong> word","age":"2 days ago"}
            ]}}"#,
        )
        .unwrap();

        let batch = parse(&value);
        assert_eq!(batch.hits[0].title, "One");
        assert_eq!(batch.hits[0].url, "https://a.test");
        assert_eq!(batch.hits[0].snippet, "a bold word");
        assert_eq!(batch.hits[0].published.as_deref(), Some("2 days ago"));
        assert_eq!(batch.cost_usd, COST_PER_QUERY);
    }
}
