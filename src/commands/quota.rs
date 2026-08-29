use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan quota` — show storage usage.
pub async fn run(client: &Client, opts: &GlobalOpts) -> Result<()> {
    let q = client.quota().await?;
    if opts.human {
        let used = crate::client::human_size(q.used);
        let total = crate::client::human_size(q.total);
        let free = crate::client::human_size(q.free);
        println!("Quota:     {} / {}", used, total);
        println!("Free:      {}", free);
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "total": q.total,
                    "used": q.used,
                    "free": q.free,
                }
            })
        );
    }
    Ok(())
}