use crate::client::{Client, UploadOptions};
use crate::commands::GlobalOpts;
use crate::error::Result;
use std::path::PathBuf;

/// `bpan upload <local> <remote>` — upload a file.
pub async fn run(
    client: &Client,
    opts: &GlobalOpts,
    local: PathBuf,
    remote: String,
    overwrite: bool,
    chunk_size: u32,
) -> Result<()> {
    let upload_opts = UploadOptions {
        overwrite,
        chunk_size_mb: chunk_size.max(1),
    };
    let result = client.upload(&local, &remote, Some(upload_opts)).await?;

    if opts.human {
        println!("✓ uploaded {} → {}", local.display(), remote);
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "local": local.display().to_string(),
                    "remote": result.path,
                    "fs_id": result.fs_id,
                    "size": result.size,
                    "md5": result.md5,
                }
            })
        );
    }
    Ok(())
}