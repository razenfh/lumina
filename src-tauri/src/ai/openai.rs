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
  let model = model.trim();

  // content: text is always sent, the image only if there is one
  let mut content = vec![json!({
    "type": "text",
    "text": prompt
  })];

  if let Some(b64) = image_base64.map(strip_data_url).filter(|s| !s.is_empty()) {
    content.push(json!({
      "type": "image_url",
      "image_url": {
        "url": format!("data:image/png;base64,{}", b64)
      }
    }));
  }

  let body = json!({
    "model": model,
    "messages": [{
      "role": "user",
      "content": content
    }]
  });

  let json = send_json(
    "OpenAI",
    client
      .post("https://api.openai.com/v1/chat/completions")
      .bearer_auth(api_key)
      .json(&body),
  )
  .await?;

  Ok(json["choices"][0]["message"]["content"]
    .as_str()
    .filter(|s| !s.is_empty())
    .unwrap_or("No response")
    .to_string())
}
