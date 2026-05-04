use std::collections::HashSet;
use std::time::Duration;

use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::{Router, routing::get};
use serde_json::json;

use crate::core::config::get_config;
use crate::core::exceptions::ApiError;
use crate::services::grok::wreq_client::body_preview;

const MODEL_SOURCE_README_URL: &str =
    "https://raw.githubusercontent.com/chenyme/grok2api/main/README.md";
const MODEL_SOURCE_TIMEOUT_SECS: u64 = 10;

pub fn router() -> Router {
    Router::new().route("/v1/models", get(list_models))
}

async fn list_models() -> Result<Response, ApiError> {
    let enabled: bool = get_config("downstream.enable_models", true).await;
    if !enabled {
        return Err(ApiError::not_found("Endpoint disabled"));
    }
    let data = fetch_model_ids_from_readme()
        .await?
        .into_iter()
        .map(|id| json!({"id": id, "object": "model", "created": 0, "owned_by": "grok2api"}))
        .collect::<Vec<_>>();
    Ok(Json(json!({"object": "list", "data": data})).into_response())
}

async fn fetch_model_ids_from_readme() -> Result<Vec<String>, ApiError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(MODEL_SOURCE_TIMEOUT_SECS))
        .build()
        .map_err(|e| ApiError::upstream(format!("Build model source client failed: {e}")))?;

    let response = client
        .get(MODEL_SOURCE_README_URL)
        .header(reqwest::header::USER_AGENT, "grok2api-rs")
        .send()
        .await
        .map_err(|e| ApiError::upstream(format!("Fetch model source failed: {e}")))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::upstream(format!("Read model source failed: {e}")))?;

    if !status.is_success() {
        let preview = body_preview(&body, 220);
        return Err(ApiError::upstream(format!(
            "Fetch model source failed: {}; body: {}",
            status.as_u16(),
            preview
        )));
    }

    let models = parse_model_ids_from_readme(&body);
    if models.is_empty() {
        return Err(ApiError::upstream(
            "No models parsed from model source README",
        ));
    }

    tracing::info!(
        source = MODEL_SOURCE_README_URL,
        count = models.len(),
        "Loaded models from README"
    );
    Ok(models)
}

fn parse_model_ids_from_readme(readme: &str) -> Vec<String> {
    let mut in_models_section = false;
    let mut seen = HashSet::new();
    let mut models = Vec::new();

    for raw_line in readme.lines() {
        let line = raw_line.trim();

        if line.starts_with("## ") {
            if line == "## 模型支持" {
                in_models_section = true;
                continue;
            }
            if in_models_section {
                break;
            }
        }

        if !in_models_section || !line.starts_with('|') {
            continue;
        }
        if line.contains(":--") || line.contains("模型名") {
            continue;
        }

        let first_cell = line
            .trim_matches('|')
            .split('|')
            .next()
            .map(str::trim)
            .unwrap_or("");
        let Some(model_id) = extract_markdown_code(first_cell) else {
            continue;
        };

        if seen.insert(model_id.clone()) {
            models.push(model_id);
        }
    }

    models
}

fn extract_markdown_code(text: &str) -> Option<String> {
    let start = text.find('`')?;
    let rest = &text[start + 1..];
    let end = rest.find('`')?;
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_model_ids_from_readme;

    #[test]
    fn parses_models_from_model_support_tables_only() {
        let readme = r#"
## 模型支持

### Chat

| 模型名 | mode | tier |
| :-- | :-- | :-- |
| `grok-a` | `fast` | `basic` |
| `grok-a` | `fast` | `basic` |

### Image

| 模型名 | mode | tier |
| :-- | :-- | :-- |
| `grok-b` | `auto` | `super` |

## API 一览

| 接口 | 是否鉴权 | 说明 |
| :-- | :-- | :-- |
| `GET /v1/models` | 是 | 列出当前启用模型 |
"#;

        assert_eq!(
            parse_model_ids_from_readme(readme),
            vec!["grok-a".to_string(), "grok-b".to_string()]
        );
    }
}
