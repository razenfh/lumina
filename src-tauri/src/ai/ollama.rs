use serde_json::json;

pub async fn ask(
  model: &str,
  prompt: &str,
  image_base64: &str,
) -> Result<String, String> {
  let client = reqwest::Client::new();

  let body = json!({
    "model": model,
    "prompt": prompt,
    "stream": false,
    "images": [image_base64]
  });

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
