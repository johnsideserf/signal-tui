use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Phone number in E.164 format (e.g., +15551234567)
    #[serde(default)]
    pub account: String,

    /// Path to signal-cli binary
    #[serde(default = "default_signal_cli_path")]
    pub signal_cli_path: String,

    /// Directory for downloaded attachments
    #[serde(default = "default_download_dir")]
    pub download_dir: PathBuf,
}

fn default_signal_cli_path() -> String {
    "signal-cli".to_string()
}

fn default_download_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("signal-downloads")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            account: String::new(),
            signal_cli_path: default_signal_cli_path(),
            download_dir: default_download_dir(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self> {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => Self::default_config_path(),
        };

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {}", config_path.display()))?;
            let config: Config = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config from {}", config_path.display()))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("signal-tui")
            .join("config.toml")
    }
}
