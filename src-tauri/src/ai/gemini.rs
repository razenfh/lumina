
use serde_json::json;

pub async fn ask(
  api_key: &str,
  model: &str,
  prompt: &str,
  image_base64: Option<&str>,
) -> Result<String, String> {
  let client = reqwest::Client::new();

  let api_key = api_key.trim();
  let model = model.trim().trim_start_matches("models/");

  let mut parts = vec![json!({ "text": prompt })];

  if let Some(b64) = image_base64 {
    let b64 = b64.trim();
    if !b64.is_empty() {
      let b64 = b64
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(b64);

      parts.push(json!({
        "inline_data": {
          "mime_type": "image/png",
          "data": b64
        }
      }));
    }
  }

  let body = json!({
    "contents": [{
      "parts": parts
    }]
  });

  let url = format!(
    "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
    model, api_key
  );

  let res = client
    .post(url)
    .json(&body)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  let status = res.status();
  let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

  if !status.is_success() {
    return Err(format!("Gemini HTTP {}: {}", status, json));
  }

  Ok(json["candidates"][0]["content"]["parts"][0]["text"]
    .as_str()
    .unwrap_or("No response")
    .to_string())
}
