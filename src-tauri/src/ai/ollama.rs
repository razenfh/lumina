use serde_json::json;

use super::{http_client, send_json, strip_data_url};

pub async fn ask(
  model: &str,
  prompt: &str,
  image_base64: Option<&str>,
) -> Result<String, String> {
  let client = http_client()?;

  let model = model.trim();

  let body = match image_base64.map(strip_data_url).filter(|s| !s.is_empty()) {
    Some(b64) => json!({
      "model": model,
      "prompt": prompt,
      "stream": false,
      "images": [b64]
    }),
    None => json!({
      "model": model,
      "prompt": prompt,
      "stream": false
    }),
  };

  let json = send_json(
    "Ollama",
    client
      .post("http://localhost:11434/api/generate")
      .json(&body),
  )
  .await?;

  Ok(json["response"].as_str().unwrap_or("").to_string())
}
