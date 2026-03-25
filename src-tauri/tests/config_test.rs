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
