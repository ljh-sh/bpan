use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan mkdir <path>` — create a directory.
pub async fn run(client: &Client, opts: &GlobalOpts, path: String) -> Result<()> {
    client.mkdir(&path).await?;
    if opts.human {
        println!("✓ mkdir {}", path);
    } else {
        println!(
            "{}",
            serde_json::json!({"ok": true, "data": {"action": "mkdir", "path": path}})
        );
    }
    Ok(())
}