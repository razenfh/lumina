#[cfg(target_os = "linux")]
use std::{
  fs,
  path::PathBuf,
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "windows")]
use std::{
  process::Command,
  time::{Duration, Instant},
};

#[cfg(any(target_os = "linux", target_os = "windows"))]
use base64::{engine::general_purpose, Engine as _};
#[cfg(target_os = "windows")]
use image::{ExtendedColorType, ImageEncoder};
use thiserror::Error;

// Variants are OS-specific, so some are never constructed on a given platform.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CaptureError {
  #[error("Spectacle not found (Linux/KDE). Install 'spectacle'.")]
  SpectacleNotFound,

  #[error("Spectacle failed: {0}")]
  SpectacleFailed(String),

  #[error("Capture was cancelled")]
  Cancelled,

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
    .map(|d| d.as_millis())
    .unwrap_or(0);

  let mut out_path: PathBuf = std::env::temp_dir();
  out_path.push(format!("lumina_capture_{}_{ts}.png", std::process::id()));

  let output = Command::new("spectacle")
    .args(["-b", "-r", "-o"])
    .arg(&out_path)
    .output()
    .map_err(|_| CaptureError::SpectacleNotFound)?;

  // Cancelling the selection can exit successfully (or not) without writing a file.
  if !output.status.success() || !out_path.exists() {
    let _ = fs::remove_file(&out_path);

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let msg = if stderr.trim().is_empty() { stdout } else { stderr };

    if msg.trim().is_empty() {
      return Err(CaptureError::Cancelled);
    }
    return Err(CaptureError::SpectacleFailed(msg.trim().to_string()));
  }

  let bytes = fs::read(&out_path).map_err(|e| CaptureError::ReadFailed(e.to_string()));
  let _ = fs::remove_file(&out_path);
  let bytes = bytes?;

  Ok(general_purpose::STANDARD.encode(bytes))
}

#[cfg(target_os = "windows")]
fn capture_windows_snip_clipboard() -> Result<String, CaptureError> {
  use arboard::Clipboard;

  let mut clipboard =
    Clipboard::new().map_err(|e| CaptureError::WindowsClipboardError(e.to_string()))?;

  // Drop whatever image is already in the clipboard; otherwise the polling loop
  // below would immediately return that stale image instead of the new snip.
  let _ = clipboard.clear();

  let opened = Command::new("cmd")
    .args(["/C", "start", "", "ms-screenclip:"])
    .spawn()
    .is_ok();

  if !opened {
    let alt = Command::new("explorer.exe")
      .arg("ms-screenclip:")
      .spawn()
      .is_ok();

    if !alt {
      return Err(CaptureError::WindowsSnipOpenFailed);
    }
  }

  // Give the user enough time to pick an area.
  let deadline = Instant::now() + Duration::from_secs(60);

  loop {
    if Instant::now() > deadline {
      return Err(CaptureError::WindowsClipboardTimeout);
    }

    if let Ok(img) = clipboard.get_image() {
      let width = img.width as u32;
      let height = img.height as u32;

      if width > 1 && height > 1 {
        let rgba = img.bytes.into_owned();

        let mut png_bytes: Vec<u8> = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);

        encoder
          .write_image(&rgba, width, height, ExtendedColorType::Rgba8)
          .map_err(|e| CaptureError::WindowsPngEncodeError(e.to_string()))?;

        return Ok(general_purpose::STANDARD.encode(png_bytes));
      }
    }

    std::thread::sleep(Duration::from_millis(50));
  }
}
