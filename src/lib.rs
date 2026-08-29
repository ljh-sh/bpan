//! # bpan
//!
//! AI-agent-first CLI + Rust SDK for Baidu Netdisk personal Open Platform.
//!
//! ## Quick start (as a library)
//!
//! ```no_run
//! use bpan::{Client, Config};
//!
//! # async fn run() -> Result<(), bpan::Error> {
//! let client = Client::new(Config::from_env()?);
//! let token = client.login_device_code().await?;
//! client.save_token(&token).await?;
//! let user = client.user_info().await?;
//! println!("hello, {}", user.baidu_name);
//! # Ok(()) }
//! ```
//!
//! ## CLI
//!
//! See [`main.rs`](../../src/main.rs) for the CLI entry point.
//!
//! ## License
//!
//! Apache-2.0. This implementation does NOT depend on Baidu's official
//! Go SDK; all HTTP calls are made directly against the documented
//! Open Platform endpoints at <https://pan.baidu.com/union/doc/基础网盘服务>.

pub mod auth;
pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod sandbox;
pub mod token;
pub mod version;

// Re-exports at crate root for convenience.
pub use crate::auth::{Auth, DeviceFlow};
pub use crate::client::Client;
pub use crate::config::Config;
pub use crate::error::{Error, Result};
pub use crate::token::AccessToken;

// Re-export key value types from client module.
pub use crate::client::{
    FileEntry, ListOptions, Quota, SearchOptions, SearchResult, UserInfo,
};