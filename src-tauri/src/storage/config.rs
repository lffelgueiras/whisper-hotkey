use crate::error::AppError;
use crate::models::app_data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "snake_case")]
pub enum OverlayPosition {
    TopCenter,
    TopLeft,
    TopRight,
    BottomCenter,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct ReplacementRule {
    pub from: String,
    pub to: String,
    pub regex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(default)]
pub struct Config {
    pub hotkey: String,
    pub auto_paste: bool,
    pub overlay_position: OverlayPosition,
    pub theme: Theme,
    pub asr_model: String,
    pub post_processing_enabled: bool,
    pub llm_model: String,
    pub llm_timeout_ms: u64,
    pub vocabulary: Vec<String>,
    pub replacements: Vec<ReplacementRule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "CmdOrControl+Shift+Space".into(),
            auto_paste: true,
            overlay_position: OverlayPosition::TopCenter,
            theme: Theme::System,
            asr_model: "whisper-base".into(),
            post_processing_enabled: false,
            llm_model: "gemma-2-2b-it-q4_k_m".into(),
            llm_timeout_ms: 8000,
            vocabulary: vec![],
            replacements: vec![],
        }
    }
}

fn config_path() -> PathBuf {
    app_data_dir().join("config.json")
}

pub fn load() -> Config {
    let p = config_path();
    match std::fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save(cfg: &Config) -> Result<(), AppError> {
    let p = config_path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(cfg)
        .map_err(|e| AppError::Internal(format!("serialize config: {e}")))?;
    std::fs::write(p, s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable() {
        let c = Config::default();
        assert_eq!(c.hotkey, "CmdOrControl+Shift+Space");
        assert!(c.auto_paste);
        assert_eq!(c.llm_timeout_ms, 8000);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let partial = r#"{"hotkey":"F12"}"#;
        let c: Config = serde_json::from_str(partial).unwrap();
        assert_eq!(c.hotkey, "F12");
        assert!(c.auto_paste);
    }
}
