//! Shared path resolution without target inspection or writes.

use std::fs;
use std::path::{Path, PathBuf};

pub fn expand_home(input: &Path, home: Option<&Path>) -> Result<PathBuf, String> {
    if input.as_os_str().as_encoded_bytes().starts_with(b"~/") {
        Ok(home
            .ok_or("HOME is required to expand ~/ in a path")?
            .join(input.strip_prefix("~").unwrap()))
    } else {
        Ok(input.to_path_buf())
    }
}

/// `repository` must be canonical. The path must exist and resolve inside it.
pub fn contained_path(repository: &Path, path: &Path, label: &str) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(path)
        .map_err(|e| format!("cannot access {label} path {}: {e}", path.display()))?;
    if !resolved.starts_with(repository) {
        return Err(format!(
            "{label} path escapes repository: {}",
            path.display()
        ));
    }
    Ok(resolved)
}

pub fn resolve_source(repository: &Path, source: &Path) -> Result<PathBuf, String> {
    if source.is_absolute() {
        return Err(format!(
            "source must be repository-relative: {}",
            source.display()
        ));
    }
    contained_path(repository, &repository.join(source), "source")
}

pub fn resolve_target(target: &Path, home: Option<&Path>) -> Result<PathBuf, String> {
    let target = expand_home(target, home)?;
    if !target.is_absolute() {
        return Err(format!(
            "target must be absolute after ~/ expansion: {}",
            target.display()
        ));
    }
    Ok(target)
}
