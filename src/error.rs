//! Unified error type for bpan.
//!
//! All public APIs return [`Error`]. CLI commands exit with codes mapped
//! from [`Error::exit_code()`].

use serde::Serialize;
use thiserror::Error;

/// bpan unified error type.
#[derive(Debug, Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(String),

    #[error("auth: {0}")]
    Auth(String),

    #[error("device code expired; user did not authorize in time")]
    DeviceCodeExpired,

    #[error("token expired; please run `bpan login` again")]
    TokenExpired,

    #[error("network: {0}")]
    Network(#[from] reqwest::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("api (errno={errno}): {message}")]
    Api { errno: i32, message: String },

    #[error("path: {0}")]
    Path(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    Permission(String),

    #[error("quota exceeded: {0}")]
    Quota(String),

    #[error("usage: {0}")]
    Usage(String),
}

impl Error {
    /// Stable error code for scripting (matches exit code 1:1).
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage(_) => 2,
            Error::Auth(_) | Error::DeviceCodeExpired | Error::TokenExpired => 3,
            Error::Permission(_) => 4,
            Error::NotFound(_) => 5,
            Error::Quota(_) => 6,
            _ => 1,
        }
    }

    /// Stable string identifier (for --json error.code field).
    pub fn code(&self) -> &'static str {
        match self {
            Error::Config(_) => "config_error",
            Error::Auth(_) => "auth_error",
            Error::DeviceCodeExpired => "device_code_expired",
            Error::TokenExpired => "token_expired",
            Error::Network(_) => "network_error",
            Error::Io(_) => "io_error",
            Error::Json(_) => "json_error",
            Error::Api { errno, .. } => match *errno {
                110 | 111 => "token_invalid",
                9019 => "share_transfer_failed",
                _ => "api_error",
            },
            Error::Path(_) => "path_error",
            Error::NotFound(_) => "not_found",
            Error::Permission(_) => "permission_denied",
            Error::Quota(_) => "quota_exceeded",
            Error::Usage(_) => "usage_error",
        }
    }

    /// Whether the caller can recover by retrying or re-authenticating.
    pub fn recoverable(&self) -> bool {
        matches!(
            self,
            Error::DeviceCodeExpired | Error::TokenExpired | Error::Auth(_) | Error::Network(_)
        )
    }
}

/// Structured error payload for --json output and agent consumption.
#[derive(Debug, Serialize)]
pub struct StructuredError<'a> {
    pub ok: bool,
    pub error: StructuredErrorInner<'a>,
}

#[derive(Debug, Serialize)]
pub struct StructuredErrorInner<'a> {
    pub code: &'a str,
    pub errno: Option<i32>,
    pub message: String,
    pub exit_code: i32,
    pub recoverable: bool,
}

impl<'a> From<&'a Error> for StructuredError<'a> {
    fn from(err: &'a Error) -> Self {
        StructuredError {
            ok: false,
            error: StructuredErrorInner {
                code: err.code(),
                errno: if let Error::Api { errno, .. } = err {
                    Some(*errno)
                } else {
                    None
                },
                message: err.to_string(),
                exit_code: err.exit_code(),
                recoverable: err.recoverable(),
            },
        }
    }
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, Error>;