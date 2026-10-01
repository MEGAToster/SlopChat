use serde_json::{json, Value};

use super::hit::{text, Batch, Hit};
use super::{BackendCtx, SearchError, TIMEOUT};

pub async fn search(query: &str, n: u32, ctx: &BackendCtx<'_>) -> Result<Batch, SearchError> {
    let body = json!({ "query": query, "limit": n });

    let value = super::send_json(
        crate::providers::client()
            .post("https://api.firecrawl.dev/v2/search")
            .header("Authorization", format!("Bearer {}", ctx.key))
            .timeout(TIMEOUT)
            .json(&body),
    )
    .await?;

    Ok(parse(&value, n))
}

fn parse(value: &Value, n: u32) -> Batch {
    // v2 returns results under `data.web`; the scrape variant returns `data` directly.
    let items = value
        .pointer("/data/web")
        .and_then(Value::as_array)
        .or_else(|| value.get("data").and_then(Value::as_array));

    let hits = items
        .map(|items| {
            items
                .iter()
                .map(|item| Hit {
                    title: text(item, "title"),
                    url: text(item, "url"),
                    snippet: text(item, "description"),
                    published: None,
                })
                .collect()
        })
        .unwrap_or_default();

    // Billing is 2 credits per 10 results.
    Batch {
        hits,
        units: (n.max(1).div_ceil(10)) * 2,
        cost_usd: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v2_web_results() {
        let value: Value = serde_json::from_str(
            r#"{"success":true,"data":{"web":[
                {"title":"One","url":"https://a.test","description":"first","position":1},
                {"title":"Two","url":"https://b.test","description":"second","position":2}
            ]}}"#,
        )
        .unwrap();

        let batch = parse(&value, 5);
        assert_eq!(batch.units, 2);
        assert_eq!(batch.hits[0].title, "One");
        assert_eq!(batch.hits[0].url, "https://a.test");
        assert_eq!(batch.hits[1].snippet, "second");
    }

    #[test]
    fn parses_scrape_variant_array() {
        let value: Value =
            serde_json::from_str(r#"{"success":true,"data":[{"title":"One","url":"https://a.test"}]}"#)
                .unwrap();
        assert_eq!(parse(&value, 10).hits.len(), 1);
    }
}
