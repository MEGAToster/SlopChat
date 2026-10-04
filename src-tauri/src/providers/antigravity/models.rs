use std::time::Duration;

use super::super::ProviderModel;
use super::binary;
use super::PROVIDER;

// `agy models` prints `slug<TAB>Display Name` lines; it has no JSON output.
pub async fn list() -> Vec<ProviderModel> {
    let Ok(path) = binary::check().await else {
        return Vec::new();
    };

    let output = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(path).arg("models").output(),
    )
    .await;

    let Ok(Ok(output)) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    parse_models(&String::from_utf8_lossy(&output.stdout))
}

fn parse_models(text: &str) -> Vec<ProviderModel> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.splitn(2, '\t');
            let id = parts.next()?.trim().to_string();
            if id.is_empty() {
                return None;
            }
            let name = parts
                .next()
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty());
            Some(ProviderModel {
                provider: PROVIDER.to_string(),
                id,
                name,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tab_separated_models() {
        let models = parse_models(include_str!("fixtures/models.txt"));
        assert!(!models.is_empty());
        assert!(models.iter().all(|model| model.provider == PROVIDER));
        assert!(models.iter().any(|model| {
            model.id == "gemini-3.8-flash-medium" && model.name.as_deref() == Some("Gemini 3.8 Flash (Medium)")
        }));
    }

    #[test]
    fn ignores_blank_and_headerless_lines() {
        assert!(parse_models("\n  \n").is_empty());
        assert_eq!(parse_models("slug-only\n")[0].name, None);
    }
}
