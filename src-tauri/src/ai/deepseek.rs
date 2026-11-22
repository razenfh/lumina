use serde_json::{json, Value};

use super::{http_client, send_json, strip_data_url};

pub async fn ask(
  api_key: &str,
  model: &str,
  prompt: &str,
  image_base64: Option<&str>,
) -> Result<String, String> {
  let client = http_client()?;

  let api_key = api_key.trim();
  let model = model.trim();

  let image = image_base64.map(strip_data_url).filter(|s| !s.is_empty());

  // Text-only requests use a plain string (what the DeepSeek API expects);
  // the multi-part form is only used when an image is attached.
  let content: Value = match image {
    Some(b64) => json!([
      { "type": "text", "text": prompt },
      {
        "type": "image_url",
        "image_url": { "url": format!("data:image/png;base64,{}", b64) }
      }
    ]),
    None => json!(prompt),
  };

  let body = json!({
    "model": model,
    "messages": [{
      "role": "user",
      "content": content
    }]
  });

  let json = send_json(
    "DeepSeek",
    client
      .post("https://api.deepseek.com/v1/chat/completions")
      .bearer_auth(api_key)
      .json(&body),
  )
  .await
  .map_err(|e| {
    if image.is_some() {
      format!(
        "{e}\nHint: this model may not support image input. Clear the screenshot or pick a vision-capable model/provider."
      )
    } else {
      e
    }
  })?;

  Ok(json["choices"][0]["message"]["content"]
    .as_str()
    .filter(|s| !s.is_empty())
    .unwrap_or("No response")
    .to_string())
}
