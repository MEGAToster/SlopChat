use serde_json::Value;

use super::hit::{text, Batch, Hit};
use super::{BackendCtx, SearchError, TIMEOUT};

const JSON_DISABLED: &str =
    "SearXNG JSON output is disabled — add `json` to search.formats in settings.yml";

pub async fn search(query: &str, n: u32, ctx: &BackendCtx<'_>) -> Result<Batch, SearchError> {
    let base = ctx.settings.searxng.base_url.trim().trim_end_matches('/');
    let mut url = reqwest::Url::parse(&format!("{base}/search"))
        .map_err(|error| SearchError::Failed(error.to_string()))?;
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("format", "json");

    let response = crate::providers::client()
        .get(url)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|error| SearchError::Failed(error.to_string()))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|error| SearchError::Failed(error.to_string()))?;

    let value = parse_response(status, &body)?;
    Ok(parse(&value, n))
}

// A public instance usually answers with HTML (or 403) when JSON is off.
fn parse_response(status: u16, body: &str) -> Result<Value, SearchError> {
    if status == 403 {
        return Err(SearchError::Failed(JSON_DISABLED.to_string()));
    }
    if !(200..300).contains(&status) {
        return Err(SearchError::Failed(format!("HTTP {status}")));
    }
    serde_json::from_str(body).map_err(|_| SearchError::Failed(JSON_DISABLED.to_string()))
}

fn parse(value: &Value, n: u32) -> Batch {
    let mut hits = Vec::new();

    // Answers and infoboxes have no link: context only, never citations.
    if let Some(answers) = value.get("answers").and_then(Value::as_array) {
        for answer in answers {
            if let Some(answer) = answer.as_str() {
                hits.push(lead("Answer", answer));
            }
        }
    }
    if let Some(infoboxes) = value.get("infoboxes").and_then(Value::as_array) {
        for infobox in infoboxes {
            let title = text(infobox, "infobox");
            let title = if title.is_empty() { "Infobox" } else { &title }.to_string();
            let content = text(infobox, "content");
            hits.push(lead(&title, &content));
        }
    }

    let results: Vec<Hit> = value
        .get("results")
        .and_then(Value::as_array)
        .map(|results| {
            results
                .iter()
                .take(n as usize)
                .map(|item| Hit {
                    title: text(item, "title"),
                    url: text(item, "url"),
                    snippet: text(item, "content"),
                    published: item
                        .get("publishedDate")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default();

    hits.extend(results);

    Batch {
        hits,
        units: 1,
        cost_usd: 0.0,
    }
}

fn lead(title: &str, snippet: &str) -> Hit {
    Hit {
        title: title.to_string(),
        url: String::new(),
        snippet: snippet.to_string(),
        published: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_results_answers_and_infoboxes() {
        let value: Value = serde_json::from_str(
            r#"{
                "answers": ["42"],
                "infoboxes": [{"infobox": "Ada Lovelace", "content": "mathematician"}],
                "results": [
                    {"title":"One","url":"https://a.test","content":"first","publishedDate":"2024-01-02"},
                    {"title":"Two","url":"https://b.test","content":"second"}
                ]
            }"#,
        )
        .unwrap();

        let batch = parse(&value, 5);
        assert_eq!(batch.units, 1);
        assert_eq!(batch.cost_usd, 0.0);
        assert_eq!(batch.hits[0].title, "Answer");
        assert!(batch.hits[0].url.is_empty());
        assert_eq!(batch.hits[1].title, "Ada Lovelace");
        assert_eq!(batch.hits[2].title, "One");
        assert_eq!(batch.hits[2].url, "https://a.test");
        assert_eq!(batch.hits[2].published.as_deref(), Some("2024-01-02"));
        assert_eq!(batch.hits[3].url, "https://b.test");
    }

    #[test]
    fn truncates_results_to_n() {
        let value: Value = serde_json::from_str(
            r#"{"results":[
                {"title":"One","url":"https://a.test","content":"first"},
                {"title":"Two","url":"https://b.test","content":"second"},
                {"title":"Three","url":"https://c.test","content":"third"}
            ]}"#,
        )
        .unwrap();

        let batch = parse(&value, 2);
        assert_eq!(batch.hits.len(), 2);
        assert_eq!(batch.hits[1].title, "Two");
    }

    #[test]
    fn classifies_403_and_html_as_json_disabled() {
        assert!(matches!(
            parse_response(403, ""),
            Err(SearchError::Failed(message)) if message.contains("JSON output is disabled")
        ));
        assert!(matches!(
            parse_response(200, "<html></html>"),
            Err(SearchError::Failed(message)) if message.contains("JSON output is disabled")
        ));
        assert!(matches!(parse_response(500, ""), Err(SearchError::Failed(_))));
        assert!(parse_response(200, r#"{"results":[]}"#).is_ok());
    }
}
