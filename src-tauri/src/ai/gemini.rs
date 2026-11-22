use serde_json::json;

use super::{http_client, send_json, strip_data_url};

pub async fn ask(
  api_key: &str,
  model: &str,
  prompt: &str,
  image_base64: Option<&str>,
) -> Result<String, String> {
  let client = http_client()?;

  let api_key = api_key.trim();
  let model = model.trim().trim_start_matches("models/");

  // The model name is placed into the URL path, so only allow plain model ids.
  if model.is_empty()
    || !model
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
  {
    return Err(format!("Invalid Gemini model name: {model:?}"));
  }

  let mut parts = vec![json!({ "text": prompt })];

  if let Some(b64) = image_base64.map(strip_data_url).filter(|s| !s.is_empty()) {
    parts.push(json!({
      "inline_data": {
        "mime_type": "image/png",
        "data": b64
      }
    }));
  }

  let body = json!({
    "contents": [{
      "parts": parts
    }]
  });

  let url = format!(
    "https://generativelanguage.googleapis.com/v1/models/{}:generateContent",
    model
  );

  // API key goes in a header (not the URL) so it can't end up in logs or error text.
  let json = send_json(
    "Gemini",
    client.post(url).header("x-goog-api-key", api_key).json(&body),
  )
  .await?;

  // A reply can consist of several parts; join all text parts.
  let text: String = json["candidates"][0]["content"]["parts"]
    .as_array()
    .map(|parts| {
      parts
        .iter()
        .filter_map(|p| p["text"].as_str())
        .collect::<Vec<_>>()
        .join("")
    })
    .unwrap_or_default();

  if !text.is_empty() {
    return Ok(text);
  }

  if let Some(reason) = json["promptFeedback"]["blockReason"].as_str() {
    return Err(format!("Gemini blocked the request: {reason}"));
  }
  if let Some(reason) = json["candidates"][0]["finishReason"].as_str() {
    return Ok(format!("No response (finish reason: {reason})"));
  }

  Ok("No response".to_string())
}
