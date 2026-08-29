use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan login` — OAuth device-code login.
pub async fn run(_client: &Client, _opts: &GlobalOpts) -> Result<()> {
    // Implementation lives in main.rs's cmd_login path because it constructs the client.
    // This file exists so commands/mod.rs can list it; main.rs handles the actual flow.
    Ok(())
}