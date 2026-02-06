# Lumina

Cross-platform AI desktop app built with **Tauri (Rust)** + **Vanilla TypeScript (Vite)**.  
A lightweight desktop client for interacting with AI models, focused on speed and simplicity.

## Features

- Cross-platform (Windows / Linux)
- Multiple AI providers (e.g. OpenAI / Gemini) via API keys
- Local settings storage (keys are stored locally on your machine)
- Screenshot / region capture workflow (when supported by OS)

> Note: You need your own API keys. This app does not provide keys.

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
Keys are stored locally on your machine.
