pub mod openai;
pub mod gemini;
pub mod deepseek;
pub mod ollama;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AiRequest {
  pub provider: String,
  pub model: String,
  pub prompt: String,
  pub image_base64: Option<String>,
}
