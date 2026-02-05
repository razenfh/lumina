mod capture;
mod ai;

use ai::AiRequest;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;

#[tauri::command]
fn capture_region() -> Result<String, String> {
  capture::capture_region_png_base64().map_err(|e| e.to_string())
}

#[tauri::command]
fn start_capture_region(app: AppHandle) -> Result<(), String> {
  std::thread::spawn(move || {
    let res = capture::capture_region_png_base64();

    match res {
      Ok(b64) => {
        let _ = app.emit("capture-done", b64);
      }
      Err(e) => {
        let _ = app.emit("capture-error", e.to_string());
      }
    }
  });

  Ok(())
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
  let image = req
    .image_base64
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty());

  match req.provider.as_str() {
    "openai" => {
      let key = get_key(&app, "openai_api_key")?;
      ai::openai::ask(&key, &req.model, &req.prompt, image).await
    }
    "gemini" => {
      let key = get_key(&app, "gemini_api_key")?;
      ai::gemini::ask(&key, &req.model, &req.prompt, image).await
    }
    "deepseek" => {
      let key = get_key(&app, "deepseek_api_key")?;
      ai::deepseek::ask(&key, &req.model, &req.prompt, image).await
    }
    "ollama" => ai::ollama::ask(&req.model, &req.prompt, image).await,
    _ => Err("Unknown provider".into()),
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_store::Builder::default().build())
    .setup(|app| {
      let store = app.store("settings.json")?;

      if store.get("provider").is_none() {
        store.set("provider", json!("openai"));
      }
      if store.get("model").is_none() {
        store.set("model", json!("gpt-4o-mini"));
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      capture_region,
      start_capture_region,
      ask_ai
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
