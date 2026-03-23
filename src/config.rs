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

#[cfg(test)]
mod tests {
    use super::*;

    // ── access_token ────────────────────────────────────────────────────

    #[test]
    fn access_token_returns_token_when_set() {
        let config = Config {
            access_token: Some("tok_abc".to_string()),
            ..Default::default()
        };
        assert_eq!(config.access_token().unwrap(), "tok_abc");
    }

    #[test]
    fn access_token_errors_when_none() {
        let config = Config::default();
        assert!(config.access_token().is_err());
    }

    #[test]
    fn access_token_errors_when_empty() {
        let config = Config {
            access_token: Some(String::new()),
            ..Default::default()
        };
        assert!(config.access_token().is_err());
    }

    // ── account_id ──────────────────────────────────────────────────────

    #[test]
    fn account_id_returns_id_when_set() {
        let config = Config {
            account_id: Some("acc_123".to_string()),
            ..Default::default()
        };
        assert_eq!(config.account_id().unwrap(), "acc_123");
    }

    #[test]
    fn account_id_errors_when_none() {
        let config = Config::default();
        assert!(config.account_id().is_err());
    }

    #[test]
    fn account_id_errors_when_empty() {
        let config = Config {
            account_id: Some(String::new()),
            ..Default::default()
        };
        assert!(config.account_id().is_err());
    }

    // ── is_token_expired ────────────────────────────────────────────────

    #[test]
    fn token_not_expired_when_no_expiry() {
        let config = Config::default();
        assert!(!config.is_token_expired());
    }

    #[test]
    fn token_not_expired_when_future() {
        let config = Config {
            token_expires_at: Some(chrono::Utc::now().timestamp() + 3600),
            ..Default::default()
        };
        assert!(!config.is_token_expired());
    }

    #[test]
    fn token_expired_when_past() {
        let config = Config {
            token_expires_at: Some(chrono::Utc::now().timestamp() - 1),
            ..Default::default()
        };
        assert!(config.is_token_expired());
    }

    // ── TOML round-trip ─────────────────────────────────────────────────

    #[test]
    fn config_toml_roundtrip() {
        let config = Config {
            client_id: Some("client_abc".to_string()),
            client_secret: Some("secret_xyz".to_string()),
            access_token: Some("tok_123".to_string()),
            refresh_token: Some("ref_456".to_string()),
            account_id: Some("acc_789".to_string()),
            user_id: Some("user_000".to_string()),
            token_expires_at: Some(1700000000),
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.client_id, config.client_id);
        assert_eq!(parsed.access_token, config.access_token);
        assert_eq!(parsed.refresh_token, config.refresh_token);
        assert_eq!(parsed.account_id, config.account_id);
        assert_eq!(parsed.token_expires_at, config.token_expires_at);
    }

    #[test]
    fn config_default_is_all_none() {
        let config = Config::default();
        assert!(config.client_id.is_none());
        assert!(config.access_token.is_none());
        assert!(config.refresh_token.is_none());
        assert!(config.account_id.is_none());
        assert!(config.token_expires_at.is_none());
    }

    #[test]
    fn config_partial_toml_parses() {
        let toml_str = r#"
            access_token = "tok_only"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.access_token, Some("tok_only".to_string()));
        assert!(config.client_id.is_none());
        assert!(config.account_id.is_none());
    }

    // ── File I/O ────────────────────────────────────────────────────────

    #[test]
    fn config_save_and_load_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config {
            access_token: Some("test_token".to_string()),
            account_id: Some("acc_test".to_string()),
            ..Default::default()
        };

        let contents = toml::to_string_pretty(&config).unwrap();
        fs::write(&path, &contents).unwrap();

        let loaded_str = fs::read_to_string(&path).unwrap();
        let loaded: Config = toml::from_str(&loaded_str).unwrap();
        assert_eq!(loaded.access_token, Some("test_token".to_string()));
        assert_eq!(loaded.account_id, Some("acc_test".to_string()));
    }
}
