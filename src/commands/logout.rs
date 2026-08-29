use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan logout` — delete saved credentials.
pub async fn run(client: &Client, opts: &GlobalOpts) -> Result<()> {
    let path = crate::config::Config::default_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
        if opts.human {
            println!("✓ Logged out — credentials deleted.");
        } else {
            println!("{}", serde_json::json!({"ok": true, "data": {"action": "logged_out"}}));
        }
    } else if opts.human {
        println!("No saved credentials.");
    } else {
        println!("{}", serde_json::json!({"ok": true, "data": {"action": "no_credentials"}}));
    }
    let _ = client;
    Ok(())
}