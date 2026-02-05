
use serde_json::json;

pub async fn ask(
  api_key: &str,
  model: &str,
  prompt: &str,
  image_base64: Option<&str>,
) -> Result<String, String> {
  let client = reqwest::Client::new();

  let api_key = api_key.trim();
  let model = model.trim();

  let mut content = vec![json!({ "type": "text", "text": prompt })];

  if let Some(b64) = image_base64 {
    let b64 = b64.trim();
    if !b64.is_empty() {
      let b64 = b64
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(b64);

      content.push(json!({
        "type": "image_url",
        "image_url": { "url": format!("data:image/png;base64,{}", b64) }
      }));
    }
  }

  let body = json!({
    "model": model,
    "messages": [{
      "role": "user",
      "content": content
    }]
  });

  let res = client
    .post("https://api.deepseek.com/v1/chat/completions")
    .bearer_auth(api_key)
    .json(&body)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  let status = res.status();
  let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

  if !status.is_success() {
    return Err(format!("DeepSeek HTTP {}: {}", status, json));
  }

  Ok(json["choices"][0]["message"]["content"]
    .as_str()
    .unwrap_or("No response")
    .to_string())
}
