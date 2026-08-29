//! HTTP client + high-level API methods.
//!
//! Wraps reqwest with bearer-token auth, retry-with-backoff, and
//! convenience methods for each Baidu Open Platform endpoint.
//!
//! The public API mirrors `baidu-netdisk/baidu-drive-sdk-go`'s scene.Scene
//! + api.Nas + api.Auth surface.

use crate::auth::Auth;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::sandbox;
use crate::token::AccessToken;
use chrono::{Duration, Utc};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client as HttpClient, Response, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration as StdDuration;

// ── Value types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserInfo {
    pub uk: i64,
    pub baidu_name: String,
    pub netdisk_name: String,
    pub avatar_url: String,
    pub vip_type: i32, // 0=normal 1=member 2=svip
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Quota {
    pub total: i64,
    pub used: i64,
    pub free: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileEntry {
    pub fs_id: u64,
    pub path: String,
    pub filename: String,
    pub is_dir: bool,
    pub size: i64,
    pub mtime: i64,
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListOptions {
    pub order: Option<String>, // "time" | "name" | "size"
    pub desc: bool,
    pub limit: u32,
    pub start: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct SearchOptions {
    pub dir: String,
    pub category: Option<Vec<i32>>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    pub fs_id: u64,
    pub path: String,
    pub filename: String,
    pub size: i64,
    pub mtime: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadOptions {
    pub overwrite: bool,
    pub chunk_size_mb: u32,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            chunk_size_mb: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadResult {
    pub fs_id: u64,
    pub path: String,
    pub size: i64,
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DownloadResult {
    pub path: String,
    pub size: i64,
    pub md5: Option<String>,
}

// ── API error envelope (returned by Baidu) ───────────────────────────

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    errno: Option<i32>,
    #[serde(default)]
    error_code: Option<i32>,
    #[serde(default)]
    errmsg: Option<String>,
    #[serde(default)]
    error_msg: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

// ── Client ──────────────────────────────────────────────────────────

/// Long-lived client. Holds Config + access token cache + HTTP client.
pub struct Client {
    config: Config,
    http: HttpClient,
    token: tokio::sync::RwLock<AccessToken>,
}

impl Client {
    /// Build a new client (does NOT call network yet).
    pub fn new(config: Config) -> Self {
        let http = HttpClient::builder()
            .user_agent(concat!("bpan/", env!("CARGO_PKG_VERSION")))
            .timeout(StdDuration::from_secs(60))
            .build()
            .expect("reqwest client builder");

        Self {
            config,
            http,
            token: tokio::sync::RwLock::new(AccessToken::default()),
        }
    }

    /// Replace the in-memory token (e.g. after login or load).
    pub async fn set_token(&self, token: AccessToken) {
        *self.token.write().await = token;
    }

    /// Get the current token (for save to disk).
    pub async fn token(&self) -> AccessToken {
        self.token.read().await.clone()
    }

    /// Reference to config (e.g. for auth flows that need client_id).
    pub fn config(&self) -> &Config {
        &self.config
    }

    // ── Auth flows (delegated to Auth) ──────────────────────────────

    /// Run OAuth device-code login. Returns the resulting token.
    pub async fn login_device_code(&self) -> Result<AccessToken> {
        let flow = Auth::device_code(self, &self.config.client_id).await?;
        eprintln!();
        eprintln!("To authorize bpan, open this URL in your browser:");
        eprintln!();
        eprintln!("    {}", flow.verification_url);
        eprintln!();
        eprintln!("And enter this code:");
        eprintln!();
        eprintln!("    {}", flow.user_code);
        eprintln!();
        eprintln!(
            "Waiting for authorization (expires at {})...",
            flow.expires_at.format("%Y-%m-%dT%H:%M:%SZ")
        );

        let token = Auth::device_token(
            self,
            &self.config.client_id,
            &self.config.client_secret,
            &flow.device_code,
        )
        .await?;
        self.set_token(token.clone()).await;
        Ok(token)
    }

    /// Save the current token to disk.
    pub async fn save_token(&self, token: &AccessToken) -> Result<()> {
        let path = crate::config::Config::default_path()?;
        let stored = crate::config::StoredConfig {
            version: 1,
            client_id: self.config.client_id.clone(),
            access_token: token.access_token.clone(),
            refresh_token: token.refresh_token.clone(),
            expires_at: token.expires_at,
            scope: token.scope.clone(),
        };
        stored.save(&path)
    }

    /// Load a previously-saved token from disk.
    pub async fn load_token(&self) -> Result<AccessToken> {
        let path = crate::config::Config::default_path()?;
        let stored = crate::config::StoredConfig::load(&path)?;
        Ok(AccessToken {
            access_token: stored.access_token,
            refresh_token: stored.refresh_token,
            expires_at: stored.expires_at,
            scope: stored.scope,
        })
    }

    /// Refresh the access token if needed (using refresh_token).
    pub async fn refresh_if_needed(&self) -> Result<()> {
        let needs = {
            let t = self.token.read().await;
            t.needs_refresh() && !t.refresh_token.is_empty()
        };
        if !needs {
            return Ok(());
        }
        let refresh_token = self.token.read().await.refresh_token.clone();
        let new = Auth::refresh_token(
            self,
            &self.config.client_id,
            &self.config.client_secret,
            &refresh_token,
        )
        .await?;
        self.set_token(new.clone()).await;
        self.save_token(&new).await?;
        Ok(())
    }

    // ── Public API methods ───────────────────────────────────────────

    pub async fn user_info(&self) -> Result<UserInfo> {
        self.refresh_if_needed().await?;
        let v: UserInfoRaw = self
            .http_get_xpan("/rest/2.0/xpan/nas", &[("method", "uinfo"), ("vip_version", "v2")])
            .await?;
        Ok(UserInfo {
            uk: v.uk,
            baidu_name: v.baidu_name,
            netdisk_name: v.netdisk_name,
            avatar_url: v.avatar_url,
            vip_type: v.vip_type,
        })
    }

    pub async fn quota(&self) -> Result<Quota> {
        self.refresh_if_needed().await?;
        #[derive(Deserialize)]
        struct Raw {
            total: i64,
            used: i64,
            free: i64,
        }
        let v: Raw = self
            .http_get_xpan("/rest/2.0/xpan/nas", &[("method", "quota")])
            .await?;
        Ok(Quota {
            total: v.total,
            used: v.used,
            free: v.free,
        })
    }

    pub async fn list_dir(&self, dir: &str, opts: Option<ListOptions>) -> Result<Vec<FileEntry>> {
        self.refresh_if_needed().await?;
        let path = sandbox::normalize_remote_path(dir)?;
        let opts = opts.unwrap_or_default();
        let order = opts.order.as_deref().unwrap_or("time");
        let desc = if opts.desc { "1" } else { "0" };
        let limit = opts.limit.max(1).to_string();
        let start = opts.start.to_string();

        #[derive(Deserialize)]
        struct RawResp {
            list: Vec<RawFile>,
        }
        #[derive(Deserialize)]
        struct RawFile {
            fs_id: u64,
            path: String,
            server_filename: String,
            isdir: u32,
            size: u64,
            server_mtime: i64,
            md5: Option<String>,
        }

        let raw: RawResp = self
            .http_get_xpan(
                "/rest/2.0/xpan/file",
                &[
                    ("method", "list"),
                    ("dir", &path),
                    ("order", order),
                    ("desc", desc),
                    ("limit", &limit),
                    ("start", &start),
                ],
            )
            .await?;

        Ok(raw
            .list
            .into_iter()
            .map(|f| FileEntry {
                fs_id: f.fs_id,
                path: f.path,
                filename: f.server_filename,
                is_dir: f.isdir == 1,
                size: f.size as i64,
                mtime: f.server_mtime,
                md5: f.md5,
            })
            .collect())
    }

    pub async fn search(&self, query: &str, opts: SearchOptions) -> Result<Vec<SearchResult>> {
        self.refresh_if_needed().await?;
        let dir = sandbox::normalize_remote_path(&opts.dir)?;
        let key = query.to_string();
        let recurse = "1".to_string();

        #[derive(Deserialize)]
        struct RawResp {
            list: Option<Vec<RawItem>>,
        }
        #[derive(Deserialize)]
        struct RawItem {
            fs_id: u64,
            path: String,
            server_filename: String,
            size: u64,
            server_mtime: i64,
        }

        let raw: RawResp = self
            .http_get_xpan(
                "/rest/2.0/xpan/file",
                &[("method", "search"), ("key", &key), ("dir", &dir), ("recursion", &recurse)],
            )
            .await?;

        Ok(raw
            .list
            .unwrap_or_default()
            .into_iter()
            .map(|r| SearchResult {
                fs_id: r.fs_id,
                path: r.path,
                filename: r.server_filename,
                size: r.size as i64,
                mtime: r.server_mtime,
            })
            .collect())
    }

    pub async fn mkdir(&self, path: &str) -> Result<()> {
        self.refresh_if_needed().await?;
        let p = sandbox::normalize_remote_path(path)?;
        let body = format!("path={}&isdir=1&size=0&block_list=[]&rtype=0", urlencoding(&p));
        let _: serde_json::Value = self
            .http_post_xpan("/rest/2.0/xpan/file", "create", &body)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, paths: &[&str]) -> Result<()> {
        self.refresh_if_needed().await?;
        let normalized: Result<Vec<String>> = paths.iter().map(|p| sandbox::normalize_remote_path(p)).collect();
        let normalized = normalized?;
        let escaped: Vec<String> = normalized.iter().map(|p| format!("\"{}\"", p)).collect();
        let body = format!("filelist=[{}]", escaped.join(","));
        let _: serde_json::Value = self
            .http_post_xpan("/rest/2.0/xpan/file", "delete", &body)
            .await?;
        Ok(())
    }

    pub async fn copy(&self, src: &str, dest_dir: &str, new_name: &str) -> Result<()> {
        self.filemanager("copy", src, dest_dir, new_name).await
    }

    pub async fn move_file(&self, src: &str, dest_dir: &str, new_name: &str) -> Result<()> {
        self.filemanager("move", src, dest_dir, new_name).await
    }

    pub async fn rename(&self, src: &str, new_name: &str) -> Result<()> {
        self.filemanager("rename", src, "/", new_name).await
    }

    async fn filemanager(
        &self,
        opera: &str,
        src: &str,
        dest_dir: &str,
        new_name: &str,
    ) -> Result<()> {
        self.refresh_if_needed().await?;
        let s = sandbox::normalize_remote_path(src)?;
        let d = sandbox::normalize_remote_path(dest_dir)?;
        let body = format!(
            "async=0&filelist=[{{\"path\":\"{}\",\"dest\":\"{}\",\"newname\":\"{}\"}}]",
            s, d, new_name
        );
        let _: serde_json::Value = self
            .http_post_xpan("/rest/2.0/xpan/file", opera, &body)
            .await?;
        Ok(())
    }

    pub async fn upload(
        &self,
        local: &Path,
        remote: &str,
        opts: Option<UploadOptions>,
    ) -> Result<UploadResult> {
        use futures_util::StreamExt;

        self.refresh_if_needed().await?;
        let opts = opts.unwrap_or_default();
        let remote = sandbox::normalize_remote_path(remote)?;
        let local_data = tokio::fs::read(local).await?;
        let size = local_data.len() as i64;
        let slice_size = (opts.chunk_size_mb as i64) * 1024 * 1024;

        // Step 1: precreate
        #[derive(Serialize)]
        struct PrecreateReq<'a> {
            path: &'a str,
            size: i64,
            isdir: i32,
            rtype: i32,
            block_list: &'a str,
        }
        #[derive(Deserialize)]
        struct PrecreateResp {
            uploadid: String,
            #[serde(default)]
            md5: Option<String>,
        }

        let precreate: PrecreateResp = self
            .http_post_xpan_json(
                "/rest/2.0/xpan/file",
                &serde_json::json!({
                    "method": "precreate",
                    "path": remote,
                    "size": size,
                    "isdir": 0,
                    "rtype": if opts.overwrite { 2 } else { 1 },
                    "block_list": serde_json::Value::Array(vec![]),
                }),
            )
            .await?;

        // Step 2: upload slices
        let token = self.token.read().await.access_token.clone();
        let upload_url = format!(
            "https://d.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&uploadid={}",
            token, precreate.uploadid
        );

        let mut offset = 0i64;
        while offset < size {
            let end = (offset + slice_size).min(size);
            let slice = &local_data[offset as usize..end as usize];
            let md5 = format!("{:x}", md5_compute(slice));
            let resp = self
                .http
                .post(&upload_url)
                .header("Content-Type", "application/octet-stream")
                .body(slice.to_vec())
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(Error::Network(reqwest::Error::from(
                    resp.error_for_status().unwrap_err(),
                )));
            }
            offset = end;
        }

        // Step 3: create file
        #[derive(Serialize)]
        struct CreateReq<'a> {
            path: &'a str,
            size: i64,
            isdir: i32,
            rtype: i32,
            uploadid: &'a str,
        }
        #[derive(Deserialize)]
        struct CreateResp {
            fs_id: u64,
            md5: Option<String>,
        }
        let create: CreateResp = self
            .http_post_xpan_json(
                "/rest/2.0/xpan/file",
                &serde_json::json!({
                    "method": "create",
                    "path": remote,
                    "size": size,
                    "isdir": 0,
                    "rtype": if opts.overwrite { 2 } else { 1 },
                    "uploadid": precreate.uploadid,
                }),
            )
            .await?;

        Ok(UploadResult {
            fs_id: create.fs_id,
            path: remote,
            size,
            md5: create.md5,
        })
    }

    pub async fn download(&self, fs_id: u64, local: &Path) -> Result<DownloadResult> {
        self.refresh_if_needed().await?;

        #[derive(Deserialize)]
        struct MetaResp {
            list: Vec<MetaFile>,
        }
        #[derive(Deserialize)]
        struct MetaFile {
            fs_id: u64,
            path: String,
            size: u64,
            md5: Option<String>,
            dlink: String,
        }

        let meta: MetaResp = self
            .http_get_xpan(
                "/rest/2.0/xpan/multimedia",
                &[("method", "filemetas"), ("fsids", &fs_id.to_string()), ("dlink", "1")],
            )
            .await?;

        let file = meta
            .list
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound(format!("fs_id {}", fs_id)))?;

        let token = self.token.read().await.access_token.clone();
        let dlink = file.dlink.replace("&dst=15", "").replace("&dst=15", "");
        let url = if dlink.contains('?') {
            format!("{}&access_token={}", dlink, token)
        } else {
            format!("{}?access_token={}", dlink, token)
        };

        let resp = self.http.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(Error::Api {
                errno: resp.status().as_u16() as i32,
                message: format!("download HTTP {}", resp.status()),
            });
        }
        let bytes = resp.bytes().await?;
        tokio::fs::write(local, &bytes).await?;

        Ok(DownloadResult {
            path: file.path,
            size: file.size as i64,
            md5: file.md5,
        })
    }

    // ── HTTP helpers ──────────────────────────────────────────────────

    /// GET to /rest/2.0/xpan/* with bearer auth.
    pub(crate) async fn http_get_xpan<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let token = self.token.read().await.access_token.clone();
        let mut url = format!("{}{}", "https://pan.baidu.com", path);
        let mut first = true;
        let mut query = String::new();
        for (k, v) in params {
            if first {
                first = false;
                query.push('?');
            } else {
                query.push('&');
            }
            query.push_str(&format!("{}={}", k, urlencoding(v)));
        }
        url.push_str(&query);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        Self::parse(resp).await
    }

    /// POST application/x-www-form-urlencoded to /rest/2.0/xpan/* with bearer auth.
    pub(crate) async fn http_post_xpan<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        method: &str,
        body: &str,
    ) -> Result<T> {
        let token = self.token.read().await.access_token.clone();
        let mut url = format!("{}{}", "https://pan.baidu.com", path);
        let body = format!("method={}&{}", method, body);

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        Self::parse(resp).await
    }

    /// POST JSON to /rest/2.0/xpan/* with bearer auth.
    pub(crate) async fn http_post_xpan_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let token = self.token.read().await.access_token.clone();
        let url = format!("{}{}", "https://pan.baidu.com", path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await?;

        Self::parse(resp).await
    }

    /// POST to non-pan.baidu.com OAuth endpoints.
    pub(crate) async fn http_post_form(&self, url: &str, body: &[(&str, &str)]) -> Result<String> {
        let resp = self
            .http
            .post(url)
            .form(body)
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(text)
    }

    async fn parse<T: serde::de::DeserializeOwned>(resp: Response) -> Result<T> {
        let status = resp.status();
        let text = resp.text().await?;

        // Try to parse as the target type first.
        if let Ok(v) = serde_json::from_str::<T>(&text) {
            return Ok(v);
        }

        // If that fails, check for an API error envelope.
        if let Ok(err) = serde_json::from_str::<ApiErrorBody>(&text) {
            let errno = err.errno.or(err.error_code).unwrap_or(status.as_u16() as i32);
            let msg = err
                .errmsg
                .or(err.error_msg)
                .or(err.error)
                .unwrap_or_else(|| text.clone());
            if errno == 110 || errno == 111 {
                return Err(Error::TokenExpired);
            }
            return Err(Error::Api {
                errno,
                message: msg,
            });
        }

        // Fall through: HTTP error without parseable body.
        if !status.is_success() {
            return Err(Error::Api {
                errno: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, text),
            });
        }

        // Body didn't deserialize and wasn't an API error.
        Err(Error::Json(
            serde_json::from_str::<serde_json::Value>(&text).unwrap_err(),
        ))
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn md5_compute(data: &[u8]) -> md5::Digest {
    md5::compute(data)
}

// ── Raw response types ──────────────────────────────────────────────

#[derive(Deserialize)]
struct UserInfoRaw {
    uk: i64,
    baidu_name: String,
    netdisk_name: String,
    avatar_url: String,
    vip_type: i32,
}

// Make md5 a dependency alias.
extern crate md5;

/// Human-readable byte size (B / KB / MB / GB / TB).
pub fn human_size(n: i64) -> String {
    const K: i64 = 1024;
    if n < K {
        return format!("{} B", n);
    }
    let mut val = n as f64 / K as f64;
    let mut unit = "KB";
    if val >= K as f64 {
        val /= K as f64;
        unit = "MB";
    }
    if val >= K as f64 {
        val /= K as f64;
        unit = "GB";
    }
    if val >= K as f64 {
        val /= K as f64;
        unit = "TB";
    }
    format!("{:.1} {}", val, unit)
}