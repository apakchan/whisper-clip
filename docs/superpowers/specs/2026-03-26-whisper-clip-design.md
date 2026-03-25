# Whisper Clip — 设计文档

## 概述

Whisper Clip 是一个 macOS 菜单栏应用，通过 Groq 的 Whisper API 将语音转为文字。用户按住全局快捷键录音，松开后自动转写，结果自动复制到剪贴板。

**核心使用场景：** 快速将语音转成文字，粘贴到 Claude Code 或其他应用中，替代缓慢的打字输入。

## 技术栈

- **框架：** Tauri v2（菜单栏 / 系统托盘模式）
- **后端：** Rust
- **前端：** 极简 HTML/JS/CSS（仅设置页面）
- **打包：** 通过 Tauri build + create-dmg 生成 DMG

## 架构

```
┌─────────────────────────────────────┐
│           Tauri 应用                 │
│                                     │
│  ┌──────────┐    ┌───────────────┐  │
│  │  前端     │    │  Rust 后端     │  │
│  │ (Web)     │    │               │  │
│  │  设置页面  │◄──►│ 全局快捷键监听  │  │
│  │  状态展示  │    │ 麦克风采集     │  │
│  │           │    │ WAV 编码      │  │
│  └──────────┘    │ Groq API 调用  │  │
│                  │ 剪贴板写入      │  │
│                  └───────────────┘  │
└─────────────────────────────────────┘
                    │
                    ▼
           ┌─────────────────┐
           │ Groq Whisper API │
           │ POST /audio/     │
           │ transcriptions   │
           └─────────────────┘
```

### Rust 核心依赖

| Crate | 用途 |
|-------|------|
| `tauri` | 应用框架，系统托盘 |
| `tauri-plugin-clipboard-manager` | 剪贴板操作 |
| `rdev` | 全局按键按下/松开事件监听（实现按住录音） |
| `cpal` | 跨平台音频采集 |
| `hound` | WAV 编码（16kHz 单声道） |
| `reqwest` | HTTP 客户端，调用 Groq API |
| `serde` / `serde_json` | 配置文件序列化 |

### 数据流

1. `rdev` 在独立线程上监听全局按键事件
2. 用户按住 `Cmd+Shift+Space` → 检测到 key-down 事件，开始录音
3. Rust 后端通过 `cpal` 以 16kHz 单声道采集麦克风音频，写入内存 buffer
4. 用户松开快捷键 → 检测到 key-up 事件，停止采集
5. 若录音时长 < 0.5 秒，丢弃并返回空闲状态
6. 通过 `hound` 将 buffer 编码为 16kHz 单声道 WAV（文件较小，约 960KB/分钟）
7. `reqwest` 以 `multipart/form-data` 格式将 WAV 文件 POST 到 Groq Whisper API
8. 解析返回的 JSON，提取转写文字
9. 写入系统剪贴板
10. 通过 Tauri 事件通知前端更新托盘图标状态

**最大录音时长：** 10 分钟（硬限制）。9:45 时图标闪烁警告，10:00 时自动停止。

### 快捷键实现说明

Tauri 的 `tauri-plugin-global-shortcut` 仅在按键按下时触发回调，不支持按键松开事件。为了实现按住录音的交互，我们使用 `rdev` crate，它提供操作系统级别的原始 `KeyPress` 和 `KeyRelease` 事件。这需要 macOS 的**辅助功能权限**（见下方系统要求）。

### 并发模型

- **主线程：** Tauri 事件循环，UI 更新
- **rdev 线程：** 独立线程监听全局按键事件
- **音频线程：** `cpal` 运行自己的回调线程进行音频采集
- **共享状态：** `Arc<Mutex<AppState>>` 持有录音标志和当前状态
- **音频 buffer：** 独立的 `Arc<Mutex<Vec<f32>>>`（或通过 `ringbuf` crate 使用无锁环形缓冲区），避免高频 cpal 回调与 UI/快捷键状态读取之间的锁竞争
- **通道通信：** `mpsc` 通道，rdev 线程 → 主线程传递按键事件
- **重叠保护：** 转写进行中时新的录音请求会排队（同一时间只有一个活跃操作）

## 菜单栏交互

### 图标状态

| 状态 | 外观 |
|------|------|
| 空闲 | 灰色麦克风图标 |
| 录音中 | 红色图标 / 脉冲动画 |
| 转写中 | 加载旋转图标 |
| 完成 | 绿色图标（约1秒），然后恢复空闲 |
| 错误 | 警告图标，短暂显示 |

### 下拉菜单

- 上次转写结果（截断显示，点击可复制完整文本）
- 快捷键提示：`Cmd+Shift+Space`
- 设置（打开设置窗口）
- 退出

### 设置窗口（极简）

- 麦克风设备选择（下拉框，默认为系统默认设备）
- 自定义快捷键绑定
- API Key 输入框（存储在本地配置文件 `~/Library/Application Support/com.whisper-clip/config.json`）
- API Key 状态：已配置 / 未配置

## API 集成

### Groq Whisper API

- **接口：** `POST https://api.groq.com/openai/v1/audio/transcriptions`
- **认证：** `Authorization: Bearer $GROQ_API_KEY`
- **请求体：** `multipart/form-data`，包含 `file`（WAV 文件）、`model`（`whisper-large-v3`）、`response_format`（`json`）
- **语言：** 自动检测（支持中英混合语音）
- **音频格式：** 16kHz 单声道 WAV（约 960KB/分钟）
- **最大文件大小：** 25 MB（Groq 限制），16kHz 单声道 WAV 约可录 26 分钟（远超我们 10 分钟的上限，10 分钟约 9.6MB）

## 错误处理

| 场景 | 行为 |
|------|------|
| API Key 未配置 | 启动时显示警告图标，下拉菜单提示"请在设置中配置 API Key" |
| 无麦克风权限 | 首次使用时 macOS 弹出权限请求；若被拒绝，通知引导至 系统设置 > 隐私与安全 > 麦克风 |
| 辅助功能权限被拒绝 | 快捷键功能不可用；通知引导至 系统设置 > 隐私与安全 > 辅助功能 |
| 录音时间过短（< 0.5秒） | 静默忽略，不发送 API 请求 |
| 网络错误 / API 失败 | 短暂显示错误图标，macOS 系统通知显示错误信息 |
| API 超时（> 30秒） | 超时，通知建议重试 |
| API 限流（429） | 通知"请求过于频繁，请稍后重试" |
| 录音达到 10 分钟上限 | 自动停止，继续进行转写 |

所有错误通过图标状态 + 系统通知处理，不弹出模态对话框，不打断用户工作流。

## 打包与分发

### DMG 构建

- `tauri build` 生成 `.app` 应用包
- `create-dmg` 或 Tauri 内置 DMG 支持生成 `.dmg` 文件
- DMG 内包含应用图标 + Applications 文件夹快捷方式，拖拽即可安装

### 系统要求

- macOS 12+（Monterey 及以上）
- 麦克风权限（`Info.plist` 中声明 `NSMicrophoneUsageDescription`）
- 辅助功能权限（`rdev` 全局按键事件监听所需）
- Groq API Key（在应用设置中配置，存储在 `~/Library/Application Support/com.whisper-clip/config.json`）

### 代码签名

- 开发阶段：不签名，用户右键点击 → 打开以绕过 Gatekeeper
- 后续正式分发时可补充签名

## 项目结构

```
whisper/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── Info.plist
│   └── src/
│       ├── main.rs         # Tauri 入口，托盘初始化
│       ├── audio.rs        # cpal 麦克风采集
│       ├── encoder.rs      # WAV 编码
│       ├── api.rs          # Groq Whisper API 客户端
│       ├── clipboard.rs    # 剪贴板操作
│       ├── hotkey.rs       # rdev 全局按键事件监听
│       ├── config.rs       # 配置文件读写（API Key、设置）
│       └── state.rs        # 共享应用状态（Arc<Mutex>）
├── src/
│   ├── index.html          # 前端入口
│   ├── main.js             # 前端逻辑
│   └── style.css           # 样式
├── icons/                  # 托盘图标（各状态）
└── docs/
```

每个 Rust 模块职责单一，文件保持小而专注。
前端极简，仅服务于设置页面和状态展示。
