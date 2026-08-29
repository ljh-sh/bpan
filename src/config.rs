//! Configuration: client_id/secret + on-disk config path.
//!
//! Reads/writes `~/.config/bdpan/config.json` (0600) for compatibility
//! with the historical Go-based bpan v0.1.0.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Schema version (always 1).
    pub version: u32,
    /// Baidu Open Platform AppKey (a.k.a. client_id).
    pub client_id: String,
    /// Baidu Open Platform SecretKey (a.k.a. client_secret).
    pub client_secret: String,
}

impl Config {
    /// Load config from env vars `BDPAN_CLIENT_ID` and `BDPAN_CLIENT_SECRET`.
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("BDPAN_CLIENT_ID").map_err(|_| {
            Error::Config("BDPAN_CLIENT_ID not set".to_string())
        })?;
        let client_secret = std::env::var("BDPAN_CLIENT_SECRET").map_err(|_| {
            Error::Config("BDPAN_CLIENT_SECRET not set".to_string())
        })?;
        Ok(Self::new(client_id, client_secret))
    }

    /// Build config from explicit values.
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            version: 1,
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }

    /// Returns the default config file path: `~/.config/bdpan/config.json`.
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| Error::Config("cannot determine home directory".to_string()))?;
        Ok(home.join(".config").join("bdpan").join("config.json"))
    }
}

/// On-disk representation of the bpan config file (includes access token).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredConfig {
    pub version: u32,
    pub client_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub scope: String,
}

impl StoredConfig {
    /// Load from disk; returns empty default if file does not exist.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(s) if !s.is_empty() => {
                let cfg: StoredConfig = serde_json::from_str(&s)?;
                Ok(cfg)
            }
            Ok(_) => Ok(Self::default()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Atomically write to disk (0600).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, PermissionsExt::from_mode(0o600))?;
        }
        // Windows: NTFS ACLs handle 0600 semantics; skip chmod.
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}