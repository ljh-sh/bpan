use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan rename <path> <newname>` — rename in place.
pub async fn run(client: &Client, opts: &GlobalOpts, path: String, new_name: String) -> Result<()> {
    client.rename(&path, &new_name).await?;
    if opts.human {
        println!("✓ rename {} → {}", path, new_name);
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {"action": "rename", "path": path, "new_name": new_name}
            })
        );
    }
    Ok(())
}