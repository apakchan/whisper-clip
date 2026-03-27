use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub hotkey: String,
    pub microphone_device: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_prompt")]
    pub prompt: String,
}

fn default_model() -> String {
    "whisper-large-v3-turbo".to_string()
}

fn default_prompt() -> String {
    "以下是一段中英文混合的语音内容。说话者是酒店行业的技术人员，可能涉及：酒店管理系统（PMS）、订房引擎（Booking Engine）、渠道管理（Channel Manager）、OTA、房态、房价、库存、预订、入住、退房、check-in、check-out、RevPAR、ADR、OCC、GDS、CRS、rate plan、room type、availability、API、微服务、数据库、部署、上线、灰度发布、Kafka、Redis、MySQL、Elasticsearch；以及 AI 与大模型相关术语如 LLM、GPT、Claude、RAG、embedding、vector database、fine-tuning、prompt engineering、Agent、MCP、token、context window、inference、Ollama、LangChain、transformer 等。".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            hotkey: "CommandLeft+ShiftLeft+Space".to_string(),
            microphone_device: None,
            model: default_model(),
            prompt: default_prompt(),
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
