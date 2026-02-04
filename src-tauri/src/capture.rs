use std::{
  fs,
  path::PathBuf,
  process::Command,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose, Engine as _};
use image::{ExtendedColorType, ImageEncoder};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
  #[error("Spectacle not found (Linux/KDE). Install 'spectacle'.")]
  SpectacleNotFound,

  #[error("Spectacle failed: {0}")]
  SpectacleFailed(String),

  #[error("Failed to read screenshot file: {0}")]
  ReadFailed(String),

  #[error("Windows: failed to open snipping UI")]
  WindowsSnipOpenFailed,

  #[error("Windows: no image found in clipboard (timeout)")]
  WindowsClipboardTimeout,

  #[error("Windows: clipboard error: {0}")]
  WindowsClipboardError(String),

  #[error("Windows: failed to encode PNG: {0}")]
  WindowsPngEncodeError(String),

  #[error("This OS is not supported yet")]
  NotSupported,
}

pub fn capture_region_png_base64() -> Result<String, CaptureError> {
  #[cfg(target_os = "linux")]
  {
    capture_linux_spectacle()
  }

  #[cfg(target_os = "windows")]
  {
    capture_windows_snip_clipboard()
  }

  #[cfg(not(any(target_os = "linux", target_os = "windows")))]
  {
    Err(CaptureError::NotSupported)
  }
}

#[cfg(target_os = "linux")]
fn capture_linux_spectacle() -> Result<String, CaptureError> {
  let ts = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();

  let mut out_path: PathBuf = std::env::temp_dir();
  out_path.push(format!("lumina_capture_{ts}.png"));

  // spectacle -b -r -o <file>
  let output = Command::new("spectacle")
    .args(["-b", "-r", "-o"])
    .arg(&out_path)
    .output()
    .map_err(|_| CaptureError::SpectacleNotFound)?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let msg = if stderr.trim().is_empty() { stdout } else { stderr };
    return Err(CaptureError::SpectacleFailed(msg));
  }

  let bytes = fs::read(&out_path).map_err(|e| CaptureError::ReadFailed(e.to_string()))?;
  let _ = fs::remove_file(&out_path);

  Ok(general_purpose::STANDARD.encode(bytes))
}

#[cfg(target_os = "windows")]
fn capture_windows_snip_clipboard() -> Result<String, CaptureError> {
  use arboard::Clipboard;

  // Открываем системный интерфейс выделения области:
  // "ms-screenclip:" — встроенный протокол Windows
  let opened = Command::new("cmd")
    .args(["/C", "start", "", "ms-screenclip:"])
    .spawn()
    .is_ok();

  if !opened {
    // альтернативный способ
    let alt = Command::new("explorer.exe")
      .arg("ms-screenclip:")
      .spawn()
      .is_ok();

    if !alt {
      return Err(CaptureError::WindowsSnipOpenFailed);
    }
  }

  // Ждём, пока пользователь выделит область и картинка появится в clipboard
  let deadline = std::time::Instant::now() + Duration::from_secs(25);

  loop {
    if std::time::Instant::now() > deadline {
      return Err(CaptureError::WindowsClipboardTimeout);
    }

    let mut cb = Clipboard::new().map_err(|e| CaptureError::WindowsClipboardError(e.to_string()))?;

    match cb.get_image() {
      Ok(img) => {
        // arboard документирует RGBA
        let width = img.width as u32;
        let height = img.height as u32;
        let rgba = img.bytes.into_owned();

        // Кодируем PNG
        let mut png_bytes: Vec<u8> = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);

        encoder
          .write_image(&rgba, width, height, ExtendedColorType::Rgba8)
          .map_err(|e: image::ImageError| CaptureError::WindowsPngEncodeError(e.to_string()))?;

        return Ok(general_purpose::STANDARD.encode(png_bytes));
      }
      Err(_) => {
        std::thread::sleep(Duration::from_millis(250));
      }
    }
  }
}
