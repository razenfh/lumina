mod capture;
mod ai;

use ai::AiRequest;
use serde_json::json;
use tauri_plugin_store::StoreExt;

#[tauri::command]
fn capture_region() -> Result<String, String> {
  capture::capture_region_png_base64().map_err(|e| e.to_string())
}

fn get_key(app: &tauri::AppHandle, key_name: &str) -> Result<String, String> {
  let store = app.store("settings.json").map_err(|e| e.to_string())?;
  let val = store.get(key_name);
  match val.and_then(|v| v.as_str().map(|s| s.to_string())) {
    Some(s) if !s.trim().is_empty() => Ok(s),
    _ => Err(format!("API key not set: {}", key_name)),
  }
}

#[tauri::command]
async fn ask_ai(app: tauri::AppHandle, req: AiRequest) -> Result<String, String> {
  match req.provider.as_str() {
    "openai" => {
      let key = get_key(&app, "openai_api_key")?;
      ai::openai::ask(&key, &req.model, &req.prompt, &req.image_base64).await
    }
    "gemini" => {
      let key = get_key(&app, "gemini_api_key")?;
      ai::gemini::ask(&key, &req.model, &req.prompt, &req.image_base64).await
    }
    "deepseek" => {
      let key = get_key(&app, "deepseek_api_key")?;
      ai::deepseek::ask(&key, &req.model, &req.prompt, &req.image_base64).await
    }
    "ollama" => {
      ai::ollama::ask(&req.model, &req.prompt, &req.image_base64).await
    }
    _ => Err("Unknown provider".into()),
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    // store plugin init :contentReference[oaicite:5]{index=5}
    .plugin(tauri_plugin_store::Builder::default().build())
    // создаём/подгружаем settings.json и кладём туда дефолты
    .setup(|app| {
      let store = app.store("settings.json")?;

      // дефолты — если пусто, будет удобнее впервые запускать
      if store.get("provider").is_none() {
        store.set("provider", json!("openai"));
      }
      if store.get("model").is_none() {
        store.set("model", json!("gpt-4o-mini"));
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![capture_region, ask_ai])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
