//! OAuth 2.0 device-code flow against Baidu Open Platform.
//!
//! This is the cleanest CLI auth flow:
//! 1. Client requests a `device_code` + `user_code`
//! 2. Client prints URL + user_code, user visits URL in browser
//! 3. Client polls `device_token` endpoint until user authorizes or code expires
//!
//! No loopback HTTP server needed (no port conflicts, no redirect_uri
//! pre-registration, SSH/headless friendly).

use crate::client::Client;
use crate::error::{Error, Result};
use crate::token::AccessToken;
use chrono::{Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Response from the device-code request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeviceFlow {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub qrcode_url: String,
    #[schemars(with = "String")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub interval: u32,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_url: String,
    qrcode_url: String,
    expires_in: i64,
    interval: u32,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: Option<String>,
    errno: Option<i32>,
    error_description: Option<String>,
}

const DEVICE_CODE_URL: &str = "https://openauth.baidu.com/oauth/2.0/device/code";
const DEVICE_TOKEN_URL: &str = "https://openauth.baidu.com/oauth/2.0/token";

/// Auth namespace — entry points for OAuth.
pub struct Auth;

impl Auth {
    /// Request a device code from Baidu Open Platform.
    pub async fn device_code(client: &Client, app_key: &str) -> Result<DeviceFlow> {
        let resp = client
            .http_post_form(DEVICE_CODE_URL, &[
                ("response_type", "device_code"),
                ("client_id", app_key),
                ("scope", "basic,netdisk"),
            ])
            .await?;

        let raw: DeviceCodeResponse = serde_json::from_str(&resp)?;
        Ok(DeviceFlow {
            device_code: raw.device_code,
            user_code: raw.user_code,
            verification_url: raw.verification_url,
            qrcode_url: raw.qrcode_url,
            expires_at: Utc::now() + Duration::seconds(raw.expires_in),
            interval: if raw.interval < 1 { 5 } else { raw.interval },
        })
    }

    /// Poll for the access token, blocking until the user authorizes
    /// or the device code expires.
    pub async fn device_token(
        client: &Client,
        app_key: &str,
        app_secret: &str,
        device_code: &str,
    ) -> Result<AccessToken> {
        // First attempt.
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let body = vec![
                ("grant_type", "device_token"),
                ("client_id", app_key),
                ("client_secret", app_secret),
                ("code", device_code),
            ];

            let resp = client.http_post_form(DEVICE_TOKEN_URL, &body).await?;
            let parsed: serde_json::Value = serde_json::from_str(&resp)?;

            if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
                match err {
                    "authorization_pending" => {
                        // Poll every 5s (respecting device.interval if larger).
                        tokio::time::sleep(Duration::seconds(5).to_std().unwrap()).await;
                        continue;
                    }
                    "expired_token" | "device_code_expired" => {
                        return Err(Error::DeviceCodeExpired);
                    }
                    "invalid_client" | "invalid_grant" => {
                        return Err(Error::Auth(format!(
                            "OAuth rejected: {} — check BDPAN_CLIENT_ID/SECRET",
                            err
                        )));
                    }
                    _ => {
                        return Err(Error::Auth(format!(
                            "unexpected OAuth error: {} — {}",
                            err,
                            parsed
                                .get("error_description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("(no description)")
                        )));
                    }
                }
            }

            let token: TokenResponse = serde_json::from_value(parsed)?;
            return Ok(AccessToken {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_at: Utc::now() + Duration::seconds(token.expires_in),
                scope: token.scope,
            });
        }
    }

    /// Exchange a refresh_token for a new access_token.
    pub async fn refresh_token(
        client: &Client,
        app_key: &str,
        app_secret: &str,
        refresh_token: &str,
    ) -> Result<AccessToken> {
        let body = vec![
            ("grant_type", "refresh_token"),
            ("client_id", app_key),
            ("client_secret", app_secret),
            ("refresh_token", refresh_token),
        ];

        let resp = client.http_post_form(DEVICE_TOKEN_URL, &body).await?;
        let parsed: serde_json::Value = serde_json::from_str(&resp)?;

        if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
            return Err(Error::TokenExpired);
        }

        let token: TokenResponse = serde_json::from_value(parsed)?;
        Ok(AccessToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at: Utc::now() + Duration::seconds(token.expires_in),
            scope: token.scope,
        })
    }
}