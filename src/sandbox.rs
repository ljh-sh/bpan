//! Path validation for Netdisk remote paths.
//!
//! Unlike `bdpan-storage` (which sandboxes to `/apps/bdpan/`), bpan
//! operates against the whole Netdisk. We only validate against
//! obviously malformed paths.

use crate::error::{Error, Result};

/// Normalize a Netdisk remote path. Returns absolute path or error.
pub fn normalize_remote_path(p: &str) -> Result<String> {
    let trimmed = p.trim();
    if trimmed.is_empty() {
        return Ok("/".to_string());
    }
    let s = if !trimmed.starts_with('/') {
        format!("/{}", trimmed)
    } else {
        trimmed.to_string()
    };

    if s.contains('\\') {
        return Err(Error::Path(format!(
            "{}: backslash not allowed in Netdisk paths",
            s
        )));
    }
    if s.chars().any(|c| c.is_control()) {
        return Err(Error::Path(format!("{}: control character", s)));
    }

    // Lexical normalization.
    let mut out: Vec<&str> = Vec::new();
    for seg in s.split('/') {
        match seg {
            "" | "." => continue,
            ".." => {
                if out.is_empty() {
                    return Err(Error::Path(format!(
                        "{}: .. above Netdisk root",
                        s
                    )));
                }
                out.pop();
            }
            other => out.push(other),
        }
    }
    if out.is_empty() {
        Ok("/".to_string())
    } else {
        Ok(format!("/{}", out.join("/")))
    }
}

/// Join a parent directory with a leaf, normalizing.
pub fn join_remote(parent: &str, leaf: &str) -> Result<String> {
    let parent = normalize_remote_path(parent)?;
    let leaf = leaf.trim_start_matches('/');
    normalize_remote_path(&format!("{}/{}", parent, leaf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize_remote_path("").unwrap(), "/");
        assert_eq!(normalize_remote_path("/").unwrap(), "/");
        assert_eq!(normalize_remote_path("/foo").unwrap(), "/foo");
        assert_eq!(normalize_remote_path("foo").unwrap(), "/foo");
        assert_eq!(normalize_remote_path("/foo/bar/").unwrap(), "/foo/bar");
        assert_eq!(normalize_remote_path("/./foo").unwrap(), "/foo");
        assert_eq!(normalize_remote_path("/foo/../bar").unwrap(), "/bar");
        assert!(normalize_remote_path("/../foo").is_err());
        assert!(normalize_remote_path("/foo\\bar").is_err());
        assert!(normalize_remote_path("/foo\x00bar").is_err());
    }
}