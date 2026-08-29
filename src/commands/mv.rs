use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan mv <src> <dst>` — move/rename.
pub async fn run(client: &Client, opts: &GlobalOpts, src: String, dst: String) -> Result<()> {
    let (src_dir, new_name) = split_parent_name(&dst);
    client.move_file(&src, &src_dir, &new_name).await?;
    if opts.human {
        println!("✓ mv {} → {}", src, dst);
    } else {
        println!(
            "{}",
            serde_json::json!({"ok": true, "data": {"action": "mv", "from": src, "to": dst}})
        );
    }
    Ok(())
}

pub(crate) fn split_parent_name(p: &str) -> (String, String) {
    if p == "/" {
        return ("/".to_string(), String::new());
    }
    match p.rfind('/') {
        Some(i) => {
            if i == 0 {
                ("/".to_string(), p[1..].to_string())
            } else {
                (p[..i].to_string(), p[i + 1..].to_string())
            }
        }
        None => (p.to_string(), String::new()),
    }
}