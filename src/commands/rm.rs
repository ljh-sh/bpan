use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan rm <path>` — delete.
pub async fn run(client: &Client, opts: &GlobalOpts, path: String) -> Result<()> {
    client.delete(&[path.as_str()]).await?;
    if opts.human {
        println!("✓ rm {}", path);
    } else {
        println!(
            "{}",
            serde_json::json!({"ok": true, "data": {"action": "rm", "path": path}})
        );
    }
    Ok(())
}