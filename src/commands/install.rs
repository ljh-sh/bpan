use crate::commands::GlobalOpts;
use crate::error::{Error, Result};
use std::path::PathBuf;

/// `bpan install` — symlink the running binary into ~/.local/bin/.
pub async fn run(opts: &GlobalOpts) -> Result<()> {
    let exe = std::env::current_exe().map_err(|e| Error::Io(e))?;
    let home = dirs::home_dir().ok_or_else(|| Error::Config("no home".to_string()))?;
    let dst: PathBuf = [home.as_path(), &PathBuf::from(".local/bin/bpan")]
        .iter()
        .collect();

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(&dst);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if let Err(e) = symlink(&exe, &dst) {
            // Fallback: copy bytes if symlink unsupported (e.g. Windows).
            let data = std::fs::read(&exe)?;
            std::fs::write(&dst, data)?;
            let _ = e;
        }
    }
    #[cfg(not(unix))]
    {
        let data = std::fs::read(&exe)?;
        std::fs::write(&dst, data)?;
    }

    if opts.human {
        println!("✓ installed bpan → {}", dst.display());
        println!("  (add {} to PATH if not already)", dst.parent().unwrap().display());
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "action": "install",
                    "binary": dst.display().to_string(),
                    "source": exe.display().to_string(),
                }
            })
        );
    }
    Ok(())
}