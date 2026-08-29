use crate::commands::GlobalOpts;
use crate::error::Result;

/// `bpan version` — print version info.
pub async fn run(opts: &GlobalOpts) -> Result<()> {
    if opts.human {
        println!("{}", crate::version::describe());
        println!("Powered by Baidu Netdisk Open Platform");
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "version": crate::version::VERSION,
                    "commit": crate::version::COMMIT,
                    "build_time": crate::version::BUILD_TIME,
                }
            })
        );
    }
    Ok(())
}