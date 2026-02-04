use serde_json::json;

pub async fn ask(
  api_key: &str,
  model: &str,
  prompt: &str,
  image_base64: &str,
) -> Result<String, String> {
  let client = reqwest::Client::new();

  let body = json!({
    "model": model,
    "messages": [{
      "role": "user",
      "content": [
        { "type": "text", "text": prompt },
        {
          "type": "image_url",
          "image_url": { "url": format!("data:image/png;base64,{}", image_base64) }
        }
      ]
    }]
  });

  let res = client
    .post("https://api.openai.com/v1/chat/completions")
    .bearer_auth(api_key)
    .json(&body)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  let status = res.status();
  let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

  if !status.is_success() {
    return Err(format!("OpenAI HTTP {}: {}", status, json));
  }

  Ok(json["choices"][0]["message"]["content"]
    .as_str()
    .unwrap_or("No response")
    .to_string())
}
