use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan cp <src> <dst>` — copy.
pub async fn run(client: &Client, opts: &GlobalOpts, src: String, dst: String) -> Result<()> {
    let (dst_dir, new_name) = super::mv::split_parent_name(&dst);
    client.copy(&src, &dst_dir, &new_name).await?;
    if opts.human {
        println!("✓ cp {} → {}", src, dst);
    } else {
        println!(
            "{}",
            serde_json::json!({"ok": true, "data": {"action": "cp", "from": src, "to": dst}})
        );
    }
    Ok(())
}