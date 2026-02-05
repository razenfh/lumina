
use serde_json::json;

pub async fn ask(
  model: &str,
  prompt: &str,
  image_base64: Option<&str>,
) -> Result<String, String> {
  let client = reqwest::Client::new();

  let model = model.trim();
  let prompt = prompt;

  let body = if let Some(b64) = image_base64.map(str::trim).filter(|s| !s.is_empty()) {
    let b64 = b64
      .strip_prefix("data:image/png;base64,")
      .unwrap_or(b64);

    json!({
      "model": model,
      "prompt": prompt,
      "stream": false,
      "images": [b64]
    })
  } else {
    json!({
      "model": model,
      "prompt": prompt,
      "stream": false
    })
  };

  let res = client
    .post("http://localhost:11434/api/generate")
    .json(&body)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  let status = res.status();
  let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

  if !status.is_success() {
    return Err(format!("Ollama HTTP {}: {}", status, json));
  }

  Ok(json["response"].as_str().unwrap_or("").to_string())
}
