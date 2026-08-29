use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::{Error, Result};
use crate::sandbox;
use std::path::PathBuf;

/// `bpan download <remote> <local>` — download a file.
pub async fn run(client: &Client, opts: &GlobalOpts, remote: String, local: PathBuf) -> Result<()> {
    let remote = sandbox::normalize_remote_path(&remote)?;
    let parent = parent_dir(&remote);
    let name = strip_prefix(&remote, &parent);
    let files = client.list_dir(&parent, None).await?;
    let entry = files
        .into_iter()
        .find(|f| f.filename == name)
        .ok_or_else(|| Error::NotFound(format!("remote file not found: {}", remote)))?;
    let result = client.download(entry.fs_id, &local).await?;

    if opts.human {
        println!("✓ downloaded {} → {}", remote, local.display());
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "remote": result.path,
                    "local": local.display().to_string(),
                    "size": result.size,
                    "md5": result.md5,
                }
            })
        );
    }
    Ok(())
}

fn parent_dir(p: &str) -> String {
    if p == "/" {
        return "/".to_string();
    }
    match p.rfind('/') {
        Some(i) if i > 0 => p[..i].to_string(),
        _ => "/".to_string(),
    }
}

fn strip_prefix(p: &str, prefix: &str) -> String {
    if prefix == "/" {
        p.trim_start_matches('/').to_string()
    } else {
        p.strip_prefix(prefix).unwrap_or(p).trim_start_matches('/').to_string()
    }
}