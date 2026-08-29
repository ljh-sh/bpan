use crate::client::Client;
use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan whoami` — show the logged-in account.
pub async fn run(client: &Client, opts: &GlobalOpts) -> Result<()> {
    let info = client.user_info().await?;
    if opts.human {
        let vip = match info.vip_type {
            0 => "normal",
            1 => "member",
            2 => "svip",
            _ => "unknown",
        };
        println!("Baidu name: {}", info.baidu_name);
        println!("Netdisk:    {}", info.netdisk_name);
        println!("UK:         {}", info.uk);
        println!("VIP type:   {}", vip);
        if !info.avatar_url.is_empty() {
            println!("Avatar:     {}", info.avatar_url);
        }
        println!();
        println!("Powered by Baidu Netdisk Open Platform");
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "uk": info.uk,
                    "baidu_name": info.baidu_name,
                    "netdisk_name": info.netdisk_name,
                    "vip_type": info.vip_type,
                    "avatar_url": info.avatar_url,
                }
            })
        );
    }
    Ok(())
}