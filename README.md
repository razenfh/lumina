# Lumina

Cross-platform AI desktop app built with **Tauri (Rust)** + **Vanilla TypeScript (Vite)**.  
A lightweight desktop client for interacting with AI models, focused on speed and simplicity.

## Features

- Cross-platform (Windows / Linux)
- Multiple AI providers: OpenAI, Gemini, DeepSeek and Ollama (local)
- Local settings storage (keys are stored locally on your machine)
- Screenshot / region capture workflow (when supported by OS)

> Note: You need your own API keys (except for Ollama, which runs locally). This app does not provide keys.

### Screenshot capture requirements

- **Linux:** requires [`spectacle`](https://apps.kde.org/spectacle/) (KDE) to be installed.
- **Windows:** uses the built-in Snipping Tool (`ms-screenclip:`).
- **macOS:** region capture is not supported yet.

### Image input

Image input depends on the model. Use a vision-capable model (e.g. `gpt-4o`, `gemini-2.5-*`, `llava` on Ollama);
text-only models such as `llama3` or `deepseek-chat` may ignore or reject the screenshot.

---

## Downloads

### Debian / Ubuntu
A **.deb package** is available in **GitHub Releases**.  
Download it from the latest release and install with:

```bash
sudo dpkg -i lumina_*.deb
```
### Arch Linux
Lumina is available in the AUR under the package name:

```bash
lumina
```
Install using your preferred AUR helper, for example:
```bash
yay -S lumina
```

## Build from source

### Requirements

- Node.js + npm
- Rust toolchain
- Tauri prerequisites for your OS:
  - Linux: `webkit2gtk`, `openssl`, build tools, etc.
  - Windows: MSVC Build Tools + WebView2

(If you’re on Arch/CachyOS, install Tauri deps via pacman; exact package names can vary by desktop environment.)

### Development

```bash
git clone https://github.com/razenfh/lumina.git
cd lumina

npm install
npm run tauri dev
```

## Configuration

The app uses a local settings store.
Add your provider API keys inside the app settings (recommended).
Keys are stored locally on your machine in `settings.json` (plain text, in the app data directory),
so keep that file private.
