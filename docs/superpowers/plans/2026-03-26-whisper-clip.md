# Whisper Clip 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 macOS 菜单栏语音转文字应用，按住快捷键录音，松开后通过 Groq Whisper API 转写并自动复制到剪贴板。

**Architecture:** Tauri v2 菜单栏应用，Rust 后端处理全部核心逻辑（录音、编码、API 调用、剪贴板），前端仅提供极简设置页面。`rdev` 监听全局按键事件实现按住录音，`cpal` 采集音频，`hound` 编码 WAV，`reqwest` 调用 Groq API。

**Tech Stack:** Tauri v2, Rust, cpal, hound, rdev, reqwest, serde/serde_json, HTML/JS/CSS

**Spec:** `docs/superpowers/specs/2026-03-26-whisper-clip-design.md`

---

## 文件结构

```
whisper/
├── src-tauri/
│   ├── Cargo.toml                 # Rust 依赖声明
│   ├── tauri.conf.json            # Tauri 配置（系统托盘、窗口、权限）
│   ├── build.rs                   # Tauri 构建脚本
│   ├── Info.plist                 # macOS 权限声明
│   ├── src/
│   │   ├── main.rs                # Tauri 入口，托盘初始化，模块连接
│   │   ├── config.rs              # 配置文件读写（API Key、麦克风、快捷键）
│   │   ├── state.rs               # 共享应用状态枚举与 AppState 结构
│   │   ├── audio.rs               # cpal 麦克风采集（开始/停止/获取 buffer）
│   │   ├── encoder.rs             # WAV 编码（f32 samples → 16kHz mono WAV bytes）
│   │   ├── api.rs                 # Groq Whisper API 客户端
│   │   ├── hotkey.rs              # rdev 全局按键事件监听
│   │   └── clipboard.rs           # 剪贴板写入封装
│   └── tests/
│       ├── config_test.rs         # 配置模块测试
│       ├── state_test.rs          # 状态模块测试
│       ├── encoder_test.rs        # WAV 编码测试
│       └── api_test.rs            # API 客户端测试
├── src/
│   ├── index.html                 # 设置页面 HTML
│   ├── main.js                    # 设置页面逻辑（Tauri IPC 调用）
│   └── style.css                  # 设置页面样式
└── icons/
    ├── icon-idle.png              # 空闲状态图标
    ├── icon-recording.png         # 录音中图标
    ├── icon-transcribing.png      # 转写中图标
    ├── icon-done.png              # 完成图标
    └── icon-error.png             # 错误图标
```

---

### Task 1: 初始化 Tauri v2 项目

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src/index.html`
- Create: `src/main.js`
- Create: `src/style.css`
- Create: `package.json`

- [ ] **Step 1: 安装 Tauri CLI**

```bash
cargo install tauri-cli@^2
```

- [ ] **Step 2: 初始化 Tauri 项目**

在 `whisper/` 目录下运行：

```bash
cargo tauri init
```

交互式配置：
- App name: `whisper-clip`
- Window title: `Whisper Clip Settings`
- Web assets path: `../src`
- Dev server URL: `../src`
- Frontend dev command: (留空)
- Frontend build command: (留空)

- [ ] **Step 3: 更新 Cargo.toml 添加所有依赖**

编辑 `src-tauri/Cargo.toml`，在 `[dependencies]` 中添加：

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rdev = "0.5"
cpal = "0.15"
hound = "3.5"
reqwest = { version = "0.12", features = ["multipart", "json"] }
tokio = { version = "1", features = ["full"] }
dirs = "6"
arboard = "3"
```

> 注意：不使用 `tauri-plugin-clipboard-manager`，直接用 `arboard` 更简洁。

- [ ] **Step 4: 创建最小前端文件**

创建 `src/index.html`：

```html
<!DOCTYPE html>
<html lang="zh">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Whisper Clip Settings</title>
  <link rel="stylesheet" href="style.css" />
</head>
<body>
  <div id="app">
    <h1>Whisper Clip</h1>
    <p>设置页面（待实现）</p>
  </div>
  <script src="main.js"></script>
</body>
</html>
```

创建 `src/style.css`：

```css
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 20px; background: #1e1e1e; color: #e0e0e0; }
h1 { font-size: 18px; margin-bottom: 12px; }
```

创建 `src/main.js`：

```javascript
// Tauri IPC will be added later
console.log("Whisper Clip Settings loaded");
```

- [ ] **Step 5: 创建最小 main.rs 入口**

编辑 `src-tauri/src/main.rs`：

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: 验证项目能编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过，无错误。

- [ ] **Step 7: 创建 .gitignore**

创建 `.gitignore`：

```
target/
node_modules/
dist/
*.dmg
.DS_Store
```

- [ ] **Step 8: 创建 package.json**

创建 `package.json`（Tauri 构建工具需要）：

```json
{
  "name": "whisper-clip",
  "private": true,
  "version": "0.1.0"
}
```

- [ ] **Step 9: Commit**

```bash
git init
git add -A
git commit -m "chore: initialize Tauri v2 project with dependencies"
```

---

### Task 2: 配置模块（config.rs）

**Files:**
- Create: `src-tauri/src/config.rs`
- Create: `src-tauri/tests/config_test.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

- [ ] **Step 1: 写失败测试 — 默认配置创建**

创建 `src-tauri/tests/config_test.rs`：

```rust
use std::path::PathBuf;
use tempfile::TempDir;

#[path = "../src/config.rs"]
mod config;

#[test]
fn test_default_config_has_empty_api_key() {
    let config = config::AppConfig::default();
    assert_eq!(config.api_key, "");
    assert_eq!(config.hotkey, "CommandLeft+ShiftLeft+Space");
    assert_eq!(config.microphone_device, None);
}

#[test]
fn test_save_and_load_config() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("config.json");

    let mut config = config::AppConfig::default();
    config.api_key = "test-key-123".to_string();

    config::save_config(&path, &config).unwrap();
    let loaded = config::load_config(&path).unwrap();

    assert_eq!(loaded.api_key, "test-key-123");
    assert_eq!(loaded.hotkey, "CommandLeft+ShiftLeft+Space");
}

#[test]
fn test_load_missing_file_returns_default() {
    let path = PathBuf::from("/tmp/nonexistent_whisper_config.json");
    let config = config::load_config(&path).unwrap();
    assert_eq!(config.api_key, "");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd src-tauri && cargo test --test config_test
```

Expected: FAIL — `config` 模块不存在。

- [ ] **Step 3: 实现 config.rs**

创建 `src-tauri/src/config.rs`：

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub hotkey: String,
    pub microphone_device: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            hotkey: "CommandLeft+ShiftLeft+Space".to_string(),
            microphone_device: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("com.whisper-clip")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config(path: &Path) -> io::Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(path)?;
    let config: AppConfig =
        serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default());
    Ok(config)
}

pub fn save_config(path: &Path, config: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)?;
    Ok(())
}
```

- [ ] **Step 4: 在 main.rs 中添加 mod 声明**

在 `src-tauri/src/main.rs` 顶部添加：

```rust
mod config;
```

- [ ] **Step 5: 添加测试依赖**

在 `src-tauri/Cargo.toml` 中添加：

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 6: 运行测试确认通过**

```bash
cd src-tauri && cargo test --test config_test
```

Expected: 3 个测试全部 PASS。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/tests/config_test.rs src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat: add config module with load/save/default"
```

---

### Task 3: 状态管理模块（state.rs）

**Files:**
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/tests/state_test.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

- [ ] **Step 1: 写失败测试**

创建 `src-tauri/tests/state_test.rs`：

```rust
#[path = "../src/state.rs"]
mod state;

#[test]
fn test_initial_state_is_idle() {
    let app_state = state::AppState::new();
    assert_eq!(app_state.status(), state::AppStatus::Idle);
    assert!(!app_state.is_recording());
}

#[test]
fn test_state_transitions() {
    let mut app_state = state::AppState::new();

    app_state.set_status(state::AppStatus::Recording);
    assert_eq!(app_state.status(), state::AppStatus::Recording);
    assert!(app_state.is_recording());

    app_state.set_status(state::AppStatus::Transcribing);
    assert_eq!(app_state.status(), state::AppStatus::Transcribing);
    assert!(!app_state.is_recording());

    app_state.set_status(state::AppStatus::Done);
    assert_eq!(app_state.status(), state::AppStatus::Done);
}

#[test]
fn test_last_transcription() {
    let mut app_state = state::AppState::new();
    assert_eq!(app_state.last_transcription(), None);

    app_state.set_last_transcription("Hello world".to_string());
    assert_eq!(app_state.last_transcription(), Some("Hello world"));
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd src-tauri && cargo test --test state_test
```

Expected: FAIL — `state` 模块不存在。

- [ ] **Step 3: 实现 state.rs**

创建 `src-tauri/src/state.rs`：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Idle,
    Recording,
    Transcribing,
    Done,
    Error,
}

pub struct AppState {
    status: AppStatus,
    last_transcription: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status: AppStatus::Idle,
            last_transcription: None,
        }
    }

    pub fn status(&self) -> AppStatus {
        self.status
    }

    pub fn set_status(&mut self, status: AppStatus) {
        self.status = status;
    }

    pub fn is_recording(&self) -> bool {
        self.status == AppStatus::Recording
    }

    pub fn last_transcription(&self) -> Option<&str> {
        self.last_transcription.as_deref()
    }

    pub fn set_last_transcription(&mut self, text: String) {
        self.last_transcription = Some(text);
    }
}
```

- [ ] **Step 4: 在 main.rs 中添加 mod 声明**

```rust
mod state;
```

- [ ] **Step 5: 运行测试确认通过**

```bash
cd src-tauri && cargo test --test state_test
```

Expected: 3 个测试全部 PASS。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/tests/state_test.rs src-tauri/src/main.rs
git commit -m "feat: add state module with AppStatus enum and AppState"
```

---

### Task 4: WAV 编码模块（encoder.rs）

**Files:**
- Create: `src-tauri/src/encoder.rs`
- Create: `src-tauri/tests/encoder_test.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

- [ ] **Step 1: 写失败测试**

创建 `src-tauri/tests/encoder_test.rs`：

```rust
#[path = "../src/encoder.rs"]
mod encoder;

#[test]
fn test_encode_silence_produces_valid_wav() {
    // 1 second of silence at 16kHz
    let samples: Vec<f32> = vec![0.0; 16000];
    let wav_bytes = encoder::encode_wav(&samples, 16000).unwrap();

    // WAV file starts with "RIFF"
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    // Contains "WAVE" format marker
    assert_eq!(&wav_bytes[8..12], b"WAVE");
    // File should be > 44 bytes (header) + sample data
    assert!(wav_bytes.len() > 44);
}

#[test]
fn test_encode_wav_correct_size() {
    // 0.5 seconds at 16kHz = 8000 samples
    let samples: Vec<f32> = vec![0.5; 8000];
    let wav_bytes = encoder::encode_wav(&samples, 16000).unwrap();

    // 16-bit mono WAV: header(44) + samples(8000 * 2 bytes) = 44 + 16000 = 16044
    assert_eq!(wav_bytes.len(), 16044);
}

#[test]
fn test_encode_empty_samples_returns_error() {
    let samples: Vec<f32> = vec![];
    let result = encoder::encode_wav(&samples, 16000);
    assert!(result.is_err());
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd src-tauri && cargo test --test encoder_test
```

Expected: FAIL — `encoder` 模块不存在。

- [ ] **Step 3: 实现 encoder.rs**

创建 `src-tauri/src/encoder.rs`：

```rust
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;

pub fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    if samples.is_empty() {
        return Err("No audio samples to encode".to_string());
    }

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    let mut writer =
        WavWriter::new(&mut buffer, spec).map_err(|e| format!("Failed to create WAV writer: {e}"))?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        writer
            .write_sample(int_sample)
            .map_err(|e| format!("Failed to write sample: {e}"))?;
    }

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV: {e}"))?;

    Ok(buffer.into_inner())
}
```

- [ ] **Step 4: 在 main.rs 中添加 mod 声明**

```rust
mod encoder;
```

- [ ] **Step 5: 运行测试确认通过**

```bash
cd src-tauri && cargo test --test encoder_test
```

Expected: 3 个测试全部 PASS。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/encoder.rs src-tauri/tests/encoder_test.rs src-tauri/src/main.rs
git commit -m "feat: add WAV encoder module (16kHz mono, 16-bit)"
```

---

### Task 5: Groq API 客户端（api.rs）

**Files:**
- Create: `src-tauri/src/api.rs`
- Create: `src-tauri/tests/api_test.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

- [ ] **Step 1: 写失败测试**

创建 `src-tauri/tests/api_test.rs`：

```rust
#[path = "../src/api.rs"]
mod api;

#[test]
fn test_build_request_url() {
    assert_eq!(
        api::GROQ_TRANSCRIPTION_URL,
        "https://api.groq.com/openai/v1/audio/transcriptions"
    );
}

#[test]
fn test_parse_transcription_response() {
    let json = r#"{"text": "Hello, world!"}"#;
    let result = api::parse_transcription_response(json).unwrap();
    assert_eq!(result, "Hello, world!");
}

#[test]
fn test_parse_invalid_response() {
    let json = r#"{"error": "something went wrong"}"#;
    let result = api::parse_transcription_response(json);
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_text_response() {
    let json = r#"{"text": ""}"#;
    let result = api::parse_transcription_response(json).unwrap();
    assert_eq!(result, "");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd src-tauri && cargo test --test api_test
```

Expected: FAIL — `api` 模块不存在。

- [ ] **Step 3: 实现 api.rs**

创建 `src-tauri/src/api.rs`：

```rust
use reqwest::multipart;
use serde::Deserialize;

pub const GROQ_TRANSCRIPTION_URL: &str =
    "https://api.groq.com/openai/v1/audio/transcriptions";

const MODEL: &str = "whisper-large-v3";
const TIMEOUT_SECS: u64 = 30;

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: Option<String>,
}

pub fn parse_transcription_response(json: &str) -> Result<String, String> {
    let resp: TranscriptionResponse =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse response: {e}"))?;
    resp.text
        .ok_or_else(|| "Response missing 'text' field".to_string())
}

pub async fn transcribe(api_key: &str, wav_bytes: Vec<u8>) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let file_part = multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to create multipart: {e}"))?;

    let form = multipart::Form::new()
        .part("file", file_part)
        .text("model", MODEL)
        .text("response_format", "json");

    let response = client
        .post(GROQ_TRANSCRIPTION_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "API request timed out (30s). Please try again.".to_string()
            } else {
                format!("Network error: {e}")
            }
        })?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("Rate limited. Please try again in a few seconds.".to_string());
    }

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, body));
    }

    parse_transcription_response(&body)
}
```

- [ ] **Step 4: 在 main.rs 中添加 mod 声明**

```rust
mod api;
```

- [ ] **Step 5: 运行测试确认通过**

```bash
cd src-tauri && cargo test --test api_test
```

Expected: 4 个测试全部 PASS。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/api.rs src-tauri/tests/api_test.rs src-tauri/src/main.rs
git commit -m "feat: add Groq Whisper API client with response parsing"
```

---

### Task 6: 音频采集模块（audio.rs）

**Files:**
- Create: `src-tauri/src/audio.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

> 注意：音频采集依赖硬件（麦克风），无法在 CI 中做完整单元测试。此模块侧重集成测试，通过手动验证。

- [ ] **Step 1: 实现 audio.rs**

创建 `src-tauri/src/audio.rs`：

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleRate, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

const TARGET_SAMPLE_RATE: u32 = 16000;
const MAX_RECORDING_SECS: u64 = 600; // 10 minutes

pub struct AudioRecorder {
    host: Host,
    device: Option<Device>,
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let host = cpal::default_host();
        Self {
            host,
            device: None,
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: TARGET_SAMPLE_RATE,
        }
    }

    pub fn list_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.name().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn select_device(&mut self, name: Option<&str>) -> Result<(), String> {
        self.device = match name {
            Some(name) => self
                .host
                .input_devices()
                .map_err(|e| format!("Failed to list devices: {e}"))?
                .find(|d| d.name().ok().as_deref() == Some(name)),
            None => self.host.default_input_device(),
        };
        if self.device.is_none() {
            return Err("No input device found".to_string());
        }
        Ok(())
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        let device = self
            .device
            .as_ref()
            .or_else(|| self.host.default_input_device())
            .ok_or("No input device available")?;

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(TARGET_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        // Check if device supports our target config, fallback to default
        let (actual_config, needs_resample) =
            match device.supported_input_configs() {
                Ok(mut configs) => {
                    let supports_target = configs.any(|c| {
                        c.channels() == 1
                            && c.min_sample_rate().0 <= TARGET_SAMPLE_RATE
                            && c.max_sample_rate().0 >= TARGET_SAMPLE_RATE
                    });
                    if supports_target {
                        (config, false)
                    } else {
                        let default_config = device
                            .default_input_config()
                            .map_err(|e| format!("Failed to get default config: {e}"))?;
                        self.sample_rate = default_config.sample_rate().0;
                        (
                            StreamConfig {
                                channels: default_config.channels(),
                                sample_rate: default_config.sample_rate(),
                                buffer_size: cpal::BufferSize::Default,
                            },
                            true,
                        )
                    }
                }
                Err(_) => (config, false),
            };

        let buffer = self.buffer.clone();
        let max_samples = (TARGET_SAMPLE_RATE as u64 * MAX_RECORDING_SECS) as usize;
        let channels = actual_config.channels as usize;
        let source_rate = actual_config.sample_rate.0;
        let target_rate = TARGET_SAMPLE_RATE;

        {
            let mut buf = buffer.lock().map_err(|e| format!("Lock error: {e}"))?;
            buf.clear();
        }

        let stream = device
            .build_input_stream(
                &actual_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buf = match buffer.lock() {
                        Ok(b) => b,
                        Err(_) => return,
                    };
                    if buf.len() >= max_samples {
                        return;
                    }

                    // Convert to mono if needed
                    let mono_samples: Vec<f32> = if channels > 1 {
                        data.chunks(channels)
                            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                            .collect()
                    } else {
                        data.to_vec()
                    };

                    // Simple downsample if needed
                    if needs_resample && source_rate != target_rate {
                        let ratio = source_rate as f64 / target_rate as f64;
                        let mut i = 0.0_f64;
                        while (i as usize) < mono_samples.len() {
                            buf.push(mono_samples[i as usize]);
                            i += ratio;
                        }
                    } else {
                        buf.extend_from_slice(&mono_samples);
                    }
                },
                |err| {
                    eprintln!("Audio stream error: {err}");
                },
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {e}"))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {e}"))?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None; // Drop stream to stop recording
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.clone()
    }

    pub fn recording_duration_secs(&self) -> f32 {
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.len() as f32 / TARGET_SAMPLE_RATE as f32
    }

    pub fn buffer_ref(&self) -> Arc<Mutex<Vec<f32>>> {
        self.buffer.clone()
    }
}
```

- [ ] **Step 2: 在 main.rs 中添加 mod 声明**

```rust
mod audio;
```

- [ ] **Step 3: 验证编译通过**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过，无错误。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio.rs src-tauri/src/main.rs
git commit -m "feat: add audio capture module with cpal (16kHz mono, 10min cap)"
```

---

### Task 7: 剪贴板模块（clipboard.rs）

**Files:**
- Create: `src-tauri/src/clipboard.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

- [ ] **Step 1: 实现 clipboard.rs**

创建 `src-tauri/src/clipboard.rs`：

```rust
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {e}"))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {e}"))?;
    Ok(())
}
```

- [ ] **Step 2: 在 main.rs 中添加 mod 声明**

```rust
mod clipboard;
```

- [ ] **Step 3: 验证编译通过**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/clipboard.rs src-tauri/src/main.rs
git commit -m "feat: add clipboard module using arboard"
```

---

### Task 8: 全局快捷键模块（hotkey.rs）

**Files:**
- Create: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/main.rs` (添加 mod 声明)

- [ ] **Step 1: 实现 hotkey.rs**

创建 `src-tauri/src/hotkey.rs`：

```rust
use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    RecordStart,
    RecordStop,
}

pub struct HotkeyListener {
    receiver: mpsc::Receiver<HotkeyEvent>,
}

impl HotkeyListener {
    /// Spawns a dedicated thread to listen for global key events.
    /// Returns a HotkeyListener that can be polled for events.
    pub fn start() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut cmd_held = false;
            let mut shift_held = false;
            let mut space_held = false;
            let mut recording = false;

            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        match key {
                            Key::MetaLeft | Key::MetaRight => cmd_held = true,
                            Key::ShiftLeft | Key::ShiftRight => shift_held = true,
                            Key::Space => {
                                if cmd_held && shift_held && !space_held {
                                    space_held = true;
                                    if !recording {
                                        recording = true;
                                        let _ = tx.send(HotkeyEvent::RecordStart);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    EventType::KeyRelease(key) => {
                        match key {
                            Key::MetaLeft | Key::MetaRight => {
                                cmd_held = false;
                                if recording {
                                    recording = false;
                                    space_held = false;
                                    let _ = tx.send(HotkeyEvent::RecordStop);
                                }
                            }
                            Key::ShiftLeft | Key::ShiftRight => {
                                shift_held = false;
                                if recording {
                                    recording = false;
                                    space_held = false;
                                    let _ = tx.send(HotkeyEvent::RecordStop);
                                }
                            }
                            Key::Space => {
                                space_held = false;
                                if recording {
                                    recording = false;
                                    let _ = tx.send(HotkeyEvent::RecordStop);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            };

            if let Err(e) = listen(callback) {
                eprintln!("rdev listen error: {:?}", e);
            }
        });

        Ok(HotkeyListener { receiver: rx })
    }

    pub fn try_recv(&self) -> Option<HotkeyEvent> {
        self.receiver.try_recv().ok()
    }

    pub fn recv(&self) -> Result<HotkeyEvent, mpsc::RecvError> {
        self.receiver.recv()
    }
}
```

- [ ] **Step 2: 在 main.rs 中添加 mod 声明**

```rust
mod hotkey;
```

- [ ] **Step 3: 验证编译通过**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hotkey.rs src-tauri/src/main.rs
git commit -m "feat: add global hotkey listener using rdev (Cmd+Shift+Space)"
```

---

### Task 9: 主模块连接 — 系统托盘与核心流程

**Files:**
- Modify: `src-tauri/src/main.rs` (完整重写)
- Modify: `src-tauri/tauri.conf.json` (托盘配置)
- Create: `icons/icon-idle.png`

- [ ] **Step 1: 创建托盘图标占位符**

```bash
mkdir -p icons
# 创建一个 22x22 的简单 PNG 占位符（后续替换为正式图标）
# 使用 ImageMagick 或手动放置一个 22x22 的 PNG 文件
```

如果没有 ImageMagick，手动放置一个 22x22 像素的 PNG 文件到 `icons/icon-idle.png`。可以从网上下载一个麦克风图标。

- [ ] **Step 2: 更新 tauri.conf.json 启用系统托盘**

编辑 `src-tauri/tauri.conf.json`，确保包含：

```json
{
  "app": {
    "withGlobalTauri": true,
    "windows": [],
    "trayIcon": {
      "iconPath": "../icons/icon-idle.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "identifier": "com.whisper-clip",
    "macOS": {
      "minimumSystemVersion": "12.0"
    }
  }
}
```

注意 `windows` 设为空数组，因为这是菜单栏应用，默认不显示窗口。

- [ ] **Step 3: 重写 main.rs 连接所有模块**

编辑 `src-tauri/src/main.rs`：

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod audio;
mod clipboard;
mod config;
mod encoder;
mod hotkey;
mod state;

use hotkey::HotkeyEvent;
use state::{AppState, AppStatus};
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, WebviewWindowBuilder, WebviewUrl,
};

fn main() {
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let recorder = Arc::new(Mutex::new(audio::AudioRecorder::new()));

    // Load config
    let config_path = config::config_path();
    let app_config = config::load_config(&config_path).unwrap_or_default();
    let api_key = Arc::new(Mutex::new(app_config.api_key.clone()));

    // Select microphone device
    {
        let mut rec = recorder.lock().unwrap();
        let _ = rec.select_device(app_config.microphone_device.as_deref());
    }

    let state_for_tray = app_state.clone();
    let recorder_for_hotkey = recorder.clone();
    let api_key_for_hotkey = api_key.clone();
    let state_for_hotkey = app_state.clone();

    tauri::Builder::default()
        .setup(move |app| {
            // Build tray menu
            let last_text_item = MenuItemBuilder::with_id("last_text", "暂无转写结果")
                .enabled(false)
                .build(app)?;
            let hotkey_hint = MenuItemBuilder::with_id("hotkey", "⌘⇧Space 按住录音")
                .enabled(false)
                .build(app)?;
            let settings_item =
                MenuItemBuilder::with_id("settings", "设置").build(app)?;
            let quit_item =
                MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&last_text_item)
                .separator()
                .item(&hotkey_hint)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Store last_text_item for dynamic updates
            let last_text_for_update = last_text_item.clone();

            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon_as_template(true)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "settings" => {
                        // Open settings window (or focus if already open)
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.set_focus();
                        } else {
                            let _ = WebviewWindowBuilder::new(
                                app,
                                "settings",
                                WebviewUrl::App("index.html".into()),
                            )
                            .title("Whisper Clip 设置")
                            .inner_size(400.0, 320.0)
                            .resizable(false)
                            .build();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    "last_text" => {
                        let st = state_for_tray.lock().unwrap();
                        if let Some(text) = st.last_transcription() {
                            let _ = clipboard::copy_to_clipboard(text);
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Start hotkey listener
            let hotkey_listener = hotkey::HotkeyListener::start()
                .map_err(|e| format!("Failed to start hotkey listener: {e}"))?;

            let app_handle = app.handle().clone();

            // Helper: reset state + icon after delay
            fn reset_state_after(
                state: Arc<Mutex<AppState>>,
                tray: tauri::tray::TrayIcon,
                from: AppStatus,
                delay_secs: u64,
            ) {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                    let mut st = state.lock().unwrap();
                    if st.status() == from {
                        st.set_status(AppStatus::Idle);
                        if let Ok(icon) = Image::from_path("icons/icon-idle.png") {
                            let _ = tray.set_icon(Some(icon));
                        }
                    }
                });
            }

            // Hotkey event processing thread
            std::thread::spawn(move || {
                loop {
                    match hotkey_listener.recv() {
                        Ok(HotkeyEvent::RecordStart) => {
                            let mut st = state_for_hotkey.lock().unwrap();
                            // Allow starting from Idle or Error (error recovery)
                            if st.status() != AppStatus::Idle
                                && st.status() != AppStatus::Error
                            {
                                continue;
                            }
                            st.set_status(AppStatus::Recording);
                            drop(st);

                            // Update tray icon to recording
                            if let Ok(icon) = Image::from_path("icons/icon-recording.png") {
                                let _ = tray.set_icon(Some(icon));
                            }

                            let mut rec = recorder_for_hotkey.lock().unwrap();
                            if let Err(e) = rec.start_recording() {
                                eprintln!("Failed to start recording: {e}");
                                let mut st = state_for_hotkey.lock().unwrap();
                                st.set_status(AppStatus::Error);
                                if let Ok(icon) = Image::from_path("icons/icon-error.png") {
                                    let _ = tray.set_icon(Some(icon));
                                }
                                reset_state_after(state_for_hotkey.clone(), tray.clone(), AppStatus::Error, 3);
                                continue;
                            }
                        }
                        Ok(HotkeyEvent::RecordStop) => {
                            let mut st = state_for_hotkey.lock().unwrap();
                            if st.status() != AppStatus::Recording {
                                continue;
                            }
                            st.set_status(AppStatus::Transcribing);
                            drop(st);

                            // Update tray icon to transcribing
                            if let Ok(icon) = Image::from_path("icons/icon-transcribing.png") {
                                let _ = tray.set_icon(Some(icon));
                            }

                            let mut rec = recorder_for_hotkey.lock().unwrap();
                            let samples = rec.stop_recording();
                            drop(rec);

                            // Check minimum duration (0.5s = 8000 samples at 16kHz)
                            if samples.len() < 8000 {
                                let mut st = state_for_hotkey.lock().unwrap();
                                st.set_status(AppStatus::Idle);
                                if let Ok(icon) = Image::from_path("icons/icon-idle.png") {
                                    let _ = tray.set_icon(Some(icon));
                                }
                                continue;
                            }

                            // Encode and transcribe
                            let key = api_key_for_hotkey.lock().unwrap().clone();
                            if key.is_empty() {
                                eprintln!("API key not configured");
                                let mut st = state_for_hotkey.lock().unwrap();
                                st.set_status(AppStatus::Error);
                                if let Ok(icon) = Image::from_path("icons/icon-error.png") {
                                    let _ = tray.set_icon(Some(icon));
                                }
                                reset_state_after(state_for_hotkey.clone(), tray.clone(), AppStatus::Error, 3);
                                continue;
                            }

                            let state_ref = state_for_hotkey.clone();
                            let tray_ref = tray.clone();
                            let last_text_ref = last_text_for_update.clone();

                            // Use Tauri's async runtime (no manual Runtime::new)
                            tauri::async_runtime::spawn(async move {
                                match encoder::encode_wav(&samples, 16000) {
                                    Ok(wav_bytes) => {
                                        match api::transcribe(&key, wav_bytes).await {
                                            Ok(text) => {
                                                let _ = clipboard::copy_to_clipboard(&text);

                                                // Update menu item with transcription (safe UTF-8 truncation)
                                                let display_text: String = if text.chars().count() > 30 {
                                                    format!("{}...", text.chars().take(30).collect::<String>())
                                                } else {
                                                    text.clone()
                                                };
                                                let _ = last_text_ref.set_text(&display_text);
                                                let _ = last_text_ref.set_enabled(true);

                                                let mut st = state_ref.lock().unwrap();
                                                st.set_last_transcription(text);
                                                st.set_status(AppStatus::Done);
                                                drop(st);

                                                // Update icon to done
                                                if let Ok(icon) = Image::from_path("icons/icon-done.png") {
                                                    let _ = tray_ref.set_icon(Some(icon));
                                                }

                                                // Reset to idle after 1 second
                                                let state_reset = state_ref.clone();
                                                let tray_reset = tray_ref.clone();
                                                tokio::time::sleep(
                                                    std::time::Duration::from_secs(1),
                                                ).await;
                                                let mut st = state_reset.lock().unwrap();
                                                if st.status() == AppStatus::Done {
                                                    st.set_status(AppStatus::Idle);
                                                    if let Ok(icon) = Image::from_path("icons/icon-idle.png") {
                                                        let _ = tray_reset.set_icon(Some(icon));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Transcription error: {e}");
                                                let mut st = state_ref.lock().unwrap();
                                                st.set_status(AppStatus::Error);
                                                drop(st);
                                                if let Ok(icon) = Image::from_path("icons/icon-error.png") {
                                                    let _ = tray_ref.set_icon(Some(icon));
                                                }
                                                // Auto-recover from error after 3s
                                                let sr = state_ref.clone();
                                                let tr = tray_ref.clone();
                                                tokio::time::sleep(
                                                    std::time::Duration::from_secs(3),
                                                ).await;
                                                let mut st = sr.lock().unwrap();
                                                if st.status() == AppStatus::Error {
                                                    st.set_status(AppStatus::Idle);
                                                    if let Ok(icon) = Image::from_path("icons/icon-idle.png") {
                                                        let _ = tr.set_icon(Some(icon));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Encoding error: {e}");
                                        let mut st = state_ref.lock().unwrap();
                                        st.set_status(AppStatus::Error);
                                        drop(st);
                                        if let Ok(icon) = Image::from_path("icons/icon-error.png") {
                                            let _ = tray_ref.set_icon(Some(icon));
                                        }
                                        // Auto-recover after 3s
                                        let sr = state_ref.clone();
                                        let tr = tray_ref.clone();
                                        tokio::time::sleep(
                                            std::time::Duration::from_secs(3),
                                        ).await;
                                        let mut st = sr.lock().unwrap();
                                        if st.status() == AppStatus::Error {
                                            st.set_status(AppStatus::Idle);
                                            if let Ok(icon) = Image::from_path("icons/icon-idle.png") {
                                                let _ = tr.set_icon(Some(icon));
                                            }
                                        }
                                    }
                                }
                            });
                        }
                        Err(_) => break,
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> **修复说明（相比 v1）:**
> - **C1:** 使用 `tauri::async_runtime::spawn` 替代手动创建 `tokio::runtime::Runtime`，避免临时 runtime 被 drop 导致异步任务丢失
> - **C4:** 存储 `last_text_item` 引用，转写成功后动态更新菜单文本并启用点击
> - **I2:** 实现设置窗口打开逻辑，使用 `WebviewWindowBuilder` 创建/聚焦窗口
> - **I3:** 每次状态变化时更新托盘图标（`tray.set_icon()`）
> - **I5:** 移除 `tauri-plugin-clipboard-manager`，只用 `arboard`
> - **I6:** Error 状态 3 秒后自动恢复 Idle，且 RecordStart 允许从 Error 状态启动

- [ ] **Step 4: 验证编译通过**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/tauri.conf.json icons/
git commit -m "feat: wire up tray menu, hotkey listener, and transcription pipeline"
```

---

### Task 10: 前端设置页面

**Files:**
- Modify: `src/index.html`
- Modify: `src/main.js`
- Modify: `src/style.css`
- Modify: `src-tauri/src/main.rs` (添加 Tauri commands)

- [ ] **Step 1: 添加 Tauri commands 到 main.rs**

在 `src-tauri/src/main.rs` 中添加 Tauri command 函数：

```rust
#[tauri::command]
fn get_config() -> Result<config::AppConfig, String> {
    let path = config::config_path();
    config::load_config(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config_cmd(new_config: config::AppConfig) -> Result<(), String> {
    let path = config::config_path();
    config::save_config(&path, &new_config).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_audio_devices() -> Vec<String> {
    let recorder = audio::AudioRecorder::new();
    recorder.list_devices()
}
```

在 `main.rs` 中更新 `invoke_handler`：

```rust
.invoke_handler(tauri::generate_handler![
    get_config,
    save_config_cmd,
    list_audio_devices,
])
```

- [ ] **Step 2: 实现设置页面 HTML**

编辑 `src/index.html`：

```html
<!DOCTYPE html>
<html lang="zh">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Whisper Clip Settings</title>
  <link rel="stylesheet" href="style.css" />
</head>
<body>
  <div id="app">
    <h1>Whisper Clip 设置</h1>

    <div class="field">
      <label for="api-key">Groq API Key</label>
      <input type="password" id="api-key" placeholder="gsk_..." />
      <span id="api-status" class="status"></span>
    </div>

    <div class="field">
      <label for="mic-device">麦克风设备</label>
      <select id="mic-device">
        <option value="">系统默认</option>
      </select>
    </div>

    <div class="field">
      <label>快捷键</label>
      <span class="hotkey-display">⌘ ⇧ Space（按住录音）</span>
    </div>

    <button id="save-btn">保存</button>
    <span id="save-status"></span>
  </div>
  <script src="main.js"></script>
</body>
</html>
```

- [ ] **Step 3: 实现设置页面样式**

编辑 `src/style.css`：

```css
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  padding: 24px;
  background: #1e1e1e;
  color: #e0e0e0;
  font-size: 14px;
}
h1 { font-size: 18px; margin-bottom: 20px; font-weight: 600; }
.field { margin-bottom: 16px; }
.field label { display: block; margin-bottom: 6px; color: #aaa; font-size: 12px; }
.field input, .field select {
  width: 100%;
  padding: 8px 10px;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 14px;
}
.field input:focus, .field select:focus {
  outline: none;
  border-color: #6b8afd;
}
.hotkey-display {
  display: inline-block;
  padding: 6px 12px;
  background: #2a2a2a;
  border-radius: 6px;
  font-family: monospace;
  font-size: 13px;
}
.status { font-size: 12px; margin-left: 8px; }
.status.ok { color: #4caf50; }
.status.error { color: #f44336; }
button {
  padding: 8px 20px;
  background: #6b8afd;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}
button:hover { background: #5a7ae6; }
#save-status { font-size: 12px; margin-left: 12px; color: #4caf50; }
```

- [ ] **Step 4: 实现设置页面逻辑**

编辑 `src/main.js`：

```javascript
const { invoke } = window.__TAURI__.core;

async function loadConfig() {
  try {
    const config = await invoke("get_config");
    document.getElementById("api-key").value = config.api_key || "";

    const status = document.getElementById("api-status");
    if (config.api_key) {
      status.textContent = "已配置";
      status.className = "status ok";
    } else {
      status.textContent = "未配置";
      status.className = "status error";
    }

    // Load audio devices
    const devices = await invoke("list_audio_devices");
    const select = document.getElementById("mic-device");
    devices.forEach((name) => {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      if (name === config.microphone_device) {
        option.selected = true;
      }
      select.appendChild(option);
    });
  } catch (e) {
    console.error("Failed to load config:", e);
  }
}

document.getElementById("save-btn").addEventListener("click", async () => {
  const apiKey = document.getElementById("api-key").value.trim();
  const micDevice = document.getElementById("mic-device").value || null;

  try {
    await invoke("save_config_cmd", {
      newConfig: {
        api_key: apiKey,
        hotkey: "CommandLeft+ShiftLeft+Space",
        microphone_device: micDevice,
      },
    });

    const saveStatus = document.getElementById("save-status");
    saveStatus.textContent = "已保存";
    setTimeout(() => { saveStatus.textContent = ""; }, 2000);

    // Update API key status
    const status = document.getElementById("api-status");
    if (apiKey) {
      status.textContent = "已配置";
      status.className = "status ok";
    } else {
      status.textContent = "未配置";
      status.className = "status error";
    }
  } catch (e) {
    console.error("Failed to save config:", e);
    const saveStatus = document.getElementById("save-status");
    saveStatus.textContent = "保存失败";
    saveStatus.style.color = "#f44336";
  }
});

loadConfig();
```

- [ ] **Step 5: 验证编译通过**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 6: Commit**

```bash
git add src/ src-tauri/src/main.rs
git commit -m "feat: add settings page with API key input, mic selection"
```

---

### Task 11: Info.plist 权限声明与打包配置

**Files:**
- Create: `src-tauri/Info.plist`
- Modify: `src-tauri/tauri.conf.json` (打包配置)

- [ ] **Step 1: 创建 Info.plist**

创建 `src-tauri/Info.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSMicrophoneUsageDescription</key>
    <string>Whisper Clip needs microphone access to record audio for speech-to-text transcription.</string>
</dict>
</plist>
```

- [ ] **Step 2: 更新 tauri.conf.json 打包配置**

确保 `src-tauri/tauri.conf.json` 的 `bundle` 部分包含：

```json
{
  "bundle": {
    "active": true,
    "identifier": "com.whisper-clip",
    "icon": ["../icons/icon-idle.png"],
    "macOS": {
      "minimumSystemVersion": "12.0",
      "infoPlist": {
        "NSMicrophoneUsageDescription": "Whisper Clip needs microphone access to record audio for speech-to-text transcription."
      }
    },
    "targets": ["dmg"]
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Info.plist src-tauri/tauri.conf.json
git commit -m "chore: add Info.plist with microphone permission, DMG build config"
```

---

### Task 12: 构建与手动测试

**Files:** 无新文件

- [ ] **Step 1: 开发模式运行**

```bash
cargo tauri dev
```

Expected: 应用启动，菜单栏出现图标。

- [ ] **Step 2: 手动测试核心流程**

测试清单：
1. 点击菜单栏图标，下拉菜单正常显示
2. 点击 Settings，设置窗口打开
3. 输入 Groq API Key 并保存
4. 按住 `Cmd+Shift+Space` 录音，松开后等待转写
5. 检查剪贴板中是否有转写文字
6. `Cmd+V` 粘贴验证

- [ ] **Step 3: 修复发现的问题**

根据手动测试结果修复 bug。

- [ ] **Step 4: Commit 修复**

```bash
git add -A
git commit -m "fix: resolve issues found during manual testing"
```

---

### Task 13: 构建 DMG

**Files:** 无新文件

- [ ] **Step 1: 执行生产构建**

```bash
cargo tauri build
```

Expected: 在 `src-tauri/target/release/bundle/dmg/` 下生成 `.dmg` 文件。

- [ ] **Step 2: 验证 DMG 安装**

1. 双击 `.dmg` 文件
2. 拖拽应用到 Applications
3. 从 Applications 启动应用
4. 验证菜单栏图标出现
5. 验证快捷键录音功能正常

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "chore: verify DMG build and packaging"
```
