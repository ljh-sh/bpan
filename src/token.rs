//! OAuth access token + refresh logic.

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessToken {
    pub access_token: String,
    pub refresh_token: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub scope: String,
}

impl AccessToken {
    /// Check if token needs refresh (within 5min safety margin).
    pub fn needs_refresh(&self) -> bool {
        Utc::now() + chrono::Duration::minutes(5) > self.expires_at
    }

    /// Check if token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

impl Default for AccessToken {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_at: Utc::now() - chrono::Duration::days(365),
            scope: String::new(),
        }
    }
}