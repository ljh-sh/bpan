//! CLI entry point for `bpan`.
//!
//! The CLI is a thin wrapper around the library (`lib.rs`). Every subcommand
//! is implemented in `commands/<name>.rs` as `async fn run(client, opts, ...)`
//! that returns `Result<()>`. JSON output is the default; `--human` switches
//! to human-readable text.

use bpan::commands::{GlobalOpts, *};
use bpan::{Client, Config};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bpan",
    version = bpan::version::VERSION,
    about = "AI-agent-first CLI for Baidu Netdisk personal Open Platform",
    long_about = None,
)]
struct Cli {
    #[command(flatten)]
    global: GlobalOpts,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// OAuth device-code login (interactive).
    Login,

    /// Delete saved credentials.
    Logout,

    /// Show the logged-in account.
    Whoami,

    /// List a Netdisk directory.
    Ls {
        /// Directory path (default `/`).
        path: Option<String>,

        #[arg(long, default_value = "50")]
        limit: u32,

        #[arg(long, default_value = "time")]
        order: String,

        #[arg(long)]
        desc: bool,
    },

    /// Upload a local file to Netdisk.
    Upload {
        local: std::path::PathBuf,
        remote: String,

        #[arg(long)]
        overwrite: bool,

        #[arg(long, default_value_t = 4)]
        chunk_size: u32,
    },

    /// Download a Netdisk file to local.
    Download {
        remote: String,
        local: std::path::PathBuf,
    },

    /// Semantic search.
    Search {
        query: String,

        #[arg(long, default_value = "/")]
        dir: String,

        #[arg(long, default_value = "all")]
        r#type: String,
    },

    /// Show storage usage.
    Quota,

    /// Create a Netdisk directory.
    Mkdir { path: String },

    /// Delete a Netdisk file or directory.
    Rm { path: String },

    /// Move or rename.
    Mv { src: String, dst: String },

    /// Copy.
    Cp { src: String, dst: String },

    /// Rename in place.
    Rename { path: String, new_name: String },

    /// Install bpan to ~/.local/bin/bpan.
    Install,

    /// Print version and exit.
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let human = cli.global.human;
    let result = run(cli).await;
    if let Err(e) = result {
        if human {
            eprintln!("bpan: {}", e);
            if let Some(hint) = hint_for(&e) {
                eprintln!("  hint: {}", hint);
            }
        } else {
            let payload = serde_json::to_string_pretty(&serde_json::json!({
                "ok": false,
                "error": {
                    "code": e.code(),
                    "message": e.to_string(),
                    "exit_code": e.exit_code(),
                    "recoverable": e.recoverable(),
                }
            }))
            .unwrap_or_else(|_| format!("{{\"ok\":false,\"error\":{}}}", e));
            println!("{}", payload);
        }
        std::process::exit(e.exit_code() as i32);
    }
}

fn hint_for(e: &bpan::Error) -> Option<String> {
    use bpan::Error::*;
    match e {
        Auth(_) | TokenExpired => Some("run `bpan login` to refresh".to_string()),
        DeviceCodeExpired => Some("device code expired; restart `bpan login`".to_string()),
        Usage(_) => Some("run `bpan --help` for usage".to_string()),
        NotFound(_) => Some("check the path; use `bpan ls /` to see what exists".to_string()),
        _ => None,
    }
}

async fn run(cli: Cli) -> bpan::Result<()> {
    let opts = cli.global.clone();

    match cli.command {
        Cmd::Version => version(&opts).await,
        Cmd::Install => install(&opts).await,
        Cmd::Logout => {
            let client = Client::new(Config::from_env()?);
            logout(&client, &opts).await
        }
        Cmd::Login => run_login(&opts).await,
        _ => {
            let client = build_client(&opts).await?;
            run_with_client(cli.command, &client, &opts).await
        }
    }
}

async fn run_login(opts: &GlobalOpts) -> bpan::Result<()> {
    let config = Config::from_env()?;
    let client = Client::new(config);
    let token = client.login_device_code().await?;
    client.save_token(&token).await?;
    if opts.human {
        println!("✓ Logged in — credentials saved to ~/.config/bdpan/config.json");
    } else {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "action": "logged_in",
                    "expires_at": token.expires_at,
                    "scope": token.scope,
                }
            })
        );
    }
    Ok(())
}

async fn build_client(opts: &GlobalOpts) -> bpan::Result<Client> {
    let config = if let Some(ref path) = opts.config {
        // Future: load Config from explicit path (config v2 not yet supported).
        Config::from_env()?
    } else {
        Config::from_env()?
    };
    let client = Client::new(config);
    // Try to load existing token.
    if let Ok(token) = client.load_token().await {
        if !token.access_token.is_empty() {
            client.set_token(token).await;
            client.refresh_if_needed().await?;
        }
    }
    Ok(client)
}

async fn run_with_client(cmd: Cmd, client: &Client, opts: &GlobalOpts) -> bpan::Result<()> {
    match cmd {
        Cmd::Whoami => whoami(client, opts).await,
        Cmd::Quota => quota(client, opts).await,
        Cmd::Ls { path, limit, order, desc } => ls(client, opts, path, limit, order, desc).await,
        Cmd::Upload { local, remote, overwrite, chunk_size } => {
            upload(client, opts, local, remote, overwrite, chunk_size).await
        }
        Cmd::Download { remote, local } => download(client, opts, remote, local).await,
        Cmd::Search { query, dir, r#type } => search(client, opts, query, dir, r#type).await,
        Cmd::Mkdir { path } => mkdir(client, opts, path).await,
        Cmd::Rm { path } => rm(client, opts, path).await,
        Cmd::Mv { src, dst } => mv(client, opts, src, dst).await,
        Cmd::Cp { src, dst } => cp(client, opts, src, dst).await,
        Cmd::Rename { path, new_name } => rename(client, opts, path, new_name).await,
        // Handled earlier
        Cmd::Login | Cmd::Logout | Cmd::Version | Cmd::Install => unreachable!(),
    }
}