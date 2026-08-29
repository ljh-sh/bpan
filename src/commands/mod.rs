//! Subcommand implementations + shared `GlobalOpts`.

use crate::error::Result;
use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct GlobalOpts {
    /// Config file path
    #[arg(long, global = true, env = "BDPAN_CONFIG")]
    pub config: Option<std::path::PathBuf>,

    /// Output human-readable text instead of JSON (default: JSON).
    #[arg(long, global = true)]
    pub human: bool,

    /// Verbose logging (writes to stderr).
    #[arg(long, global = true, short = 'v')]
    pub verbose: bool,

    /// Disable ANSI color (default: respect NO_COLOR env).
    #[arg(long, global = true)]
    pub no_color: bool,
}

pub mod cp;
pub mod download;
pub mod install;
pub mod login;
pub mod logout;
pub mod ls;
pub mod mkdir;
pub mod mv;
pub mod quota;
pub mod rename;
pub mod rm;
pub mod search;
pub mod upload;
pub mod version;
pub mod whoami;

// Re-export for ergonomic use in main.rs.
pub use cp::run as cp;
pub use download::run as download;
pub use install::run as install;
pub use login::run as login;
pub use logout::run as logout;
pub use ls::run as ls;
pub use mkdir::run as mkdir;
pub use mv::run as mv;
pub use quota::run as quota;
pub use rename::run as rename;
pub use rm::run as rm;
pub use search::run as search;
pub use upload::run as upload;
pub use version::run as version;
pub use whoami::run as whoami;

pub type CmdResult = Result<()>;