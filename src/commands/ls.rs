use crate::client::{Client, ListOptions};
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan ls [path]` — list a Netdisk directory.
pub async fn run(client: &Client, opts: &GlobalOpts, path: Option<String>, limit: u32, order: String, desc: bool) -> Result<()> {
    let p = path.as_deref().unwrap_or("/");
    let opts_list = ListOptions {
        order: Some(order),
        desc,
        limit: limit.max(1),
        start: 0,
    };
    let entries = client.list_dir(p, Some(opts_list)).await?;

    if opts.human {
        if entries.is_empty() {
            println!("(empty directory {})", p);
            return Ok(());
        }
        for f in &entries {
            let kind = if f.is_dir { "d" } else { "-" };
            let size = crate::client::human_size(f.size);
            // Convert unix mtime to readable
            let mtime = chrono::DateTime::<chrono::Utc>::from_timestamp(f.mtime, 0)
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "?".to_string());
            println!("{}\t{}\t{}\t{}", kind, size, mtime, f.filename);
        }
        println!("total: {} entries", entries.len());
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "path": p,
                    "entries": entries,
                    "total": entries.len(),
                }
            })
        );
    }
    Ok(())
}