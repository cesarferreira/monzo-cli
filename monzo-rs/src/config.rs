use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub account_id: Option<String>,
    pub user_id: Option<String>,
    /// Unix timestamp when the access token expires
    pub token_expires_at: Option<i64>,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "cesarferreira", "monzo-cli")
            .context("could not determine config directory")?;
        let dir = dirs.config_dir().to_path_buf();
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| "failed to parse config")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    pub fn access_token(&self) -> Result<&str> {
        self.access_token.as_deref().filter(|t| !t.is_empty()).ok_or_else(|| {
            anyhow::anyhow!(
                "No access token configured. Run `monzo-cli auth login` to authenticate."
            )
        })
    }

    pub fn account_id(&self) -> Result<&str> {
        self.account_id.as_deref().filter(|t| !t.is_empty()).ok_or_else(|| {
            anyhow::anyhow!(
                "No account ID configured. Run `monzo-cli auth login` or set it in the config."
            )
        })
    }

    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(exp) => chrono::Utc::now().timestamp() >= exp,
            None => false, // no expiry info means we assume it's valid
        }
    }
}
