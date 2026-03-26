# Whisper Clip

macOS menu bar speech-to-text app. Hold a hotkey to record, release to transcribe via [Groq Whisper API](https://console.groq.com/), result auto-copied to clipboard.

## Features

- **Press-and-hold recording** — `Cmd+Shift+Space` to record, release to stop
- **Groq Whisper API** — fast, accurate transcription (whisper-large-v3-turbo)
- **Chinese-English mixed speech** — optimized for bilingual dictation
- **Menu bar app** — lives in the system tray, no dock icon
- **Sound feedback** — Glass chime on record start, Tink on transcription complete
- **Icon status** — tray icon changes to reflect recording / transcribing / done / error

## Requirements

- macOS 12+ (Monterey)
- [Groq API Key](https://console.groq.com/keys)
- Accessibility permission (for global hotkey)
- Microphone permission

## Install from source

```bash
# Install Rust and Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli@^2

# Clone and build
git clone https://github.com/apakchan/whisper-clip.git
cd whisper-clip
cargo tauri build
```

DMG output: `src-tauri/target/release/bundle/dmg/whisper-clip_*.dmg`

## Setup

1. Open the DMG, drag to Applications
2. Right-click the app → Open (first launch, bypasses Gatekeeper)
3. Grant **Accessibility** permission in System Settings > Privacy & Security > Accessibility
4. Grant **Microphone** permission when prompted
5. Click the menu bar icon → Settings → enter your Groq API Key → Save

## Usage

1. Hold `Cmd+Shift+Space` — hear Glass chime, start recording
2. Speak (Chinese, English, or mixed)
3. Release keys — transcription starts
4. Hear Tink — text is in your clipboard, `Cmd+V` to paste

## Tech Stack

- [Tauri v2](https://v2.tauri.app/) — app framework
- Rust — audio capture (cpal), WAV encoding (hound), API client (reqwest)
- macOS CGEventTap — global hotkey (press-and-hold)
- Groq Whisper API — speech-to-text
