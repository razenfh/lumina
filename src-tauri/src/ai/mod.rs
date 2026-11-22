pub mod openai;
pub mod gemini;
pub mod deepseek;
pub mod ollama;

use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AiRequest {
  pub provider: String,
  pub model: String,
  pub prompt: String,
  pub image_base64: Option<String>,
}

/// HTTP client with sane timeouts, so a stalled provider can't hang the UI forever.
pub fn http_client() -> Result<reqwest::Client, String> {
  reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(15))
    .timeout(Duration::from_secs(120))
    .build()
    .map_err(|e| e.to_string())
}

/// Accepts either raw base64 or a `data:image/png;base64,...` URL and returns raw base64.
pub fn strip_data_url(b64: &str) -> &str {
  let b64 = b64.trim();
  b64.strip_prefix("data:image/png;base64,").unwrap_or(b64)
}

/// Sends the request and returns the parsed JSON body.
///
/// Error messages never contain the request URL (reqwest includes it by default),
/// so API keys passed in a URL can't leak into the UI. Non-JSON error bodies
/// (e.g. an HTML 502 page) are reported with their HTTP status instead of a
/// confusing "error decoding response body".
pub async fn send_json(
  provider: &str,
  req: reqwest::RequestBuilder,
) -> Result<Value, String> {
  let res = req.send().await.map_err(|e| {
    let e = e.without_url();
    if e.is_timeout() {
      format!("{provider}: request timed out")
    } else if e.is_connect() {
      format!("{provider}: could not connect — check that the server is reachable / running and your network connection")
    } else {
      format!("{provider}: request failed: {e}")
    }
  })?;

  let status = res.status();
  let text = res
    .text()
    .await
    .map_err(|e| format!("{provider}: failed to read response: {}", e.without_url()))?;

  let json: Option<Value> = serde_json::from_str(&text).ok();

  if !status.is_success() {
    let detail = json.map(|j| j.to_string()).unwrap_or(text);
    let detail: String = detail.chars().take(1000).collect();
    return Err(format!("{provider} HTTP {status}: {detail}"));
  }

  json.ok_or_else(|| format!("{provider}: response was not valid JSON"))
}
