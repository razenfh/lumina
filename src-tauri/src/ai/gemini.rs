use serde_json::json;

pub async fn ask(
  api_key: &str,
  model: &str,
  prompt: &str,
  image_base64: &str,
) -> Result<String, String> {
  let client = reqwest::Client::new();

  let api_key = api_key.trim();
  let model = model.trim().trim_start_matches("models/");

  let image_base64 = image_base64.trim();
  let image_base64 = image_base64
    .strip_prefix("data:image/png;base64,")
    .unwrap_or(image_base64);

  let body = json!({
    "contents": [{
      "parts": [
        { "text": prompt },
        {
          "inline_data": {
            "mime_type": "image/png",
            "data": image_base64
          }
        }
      ]
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
