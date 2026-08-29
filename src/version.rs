//! Build-time version metadata, stamped via -ldflags at link time.

/// Semantic version (e.g. "0.2.0"), set from the git tag.
pub const VERSION: &str = match option_env!("BPAN_VERSION") {
    Some(v) => v,
    None => "dev",
};

/// Git commit SHA at build time.
pub const COMMIT: &str = match option_env!("BPAN_COMMIT") {
    Some(c) => c,
    None => "unknown",
};

/// RFC3339 build timestamp.
pub const BUILD_TIME: &str = match option_env!("BPAN_BUILD_TIME") {
    Some(t) => t,
    None => "unknown",
};

/// One-line human-readable summary.
pub fn describe() -> String {
    format!(
        "bpan {} (commit {}, built {})",
        VERSION, COMMIT, BUILD_TIME
    )
}