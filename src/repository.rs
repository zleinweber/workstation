//! Shared repository selection and recipe discovery. These operations only read files.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{Recipe, parse_recipe};

pub fn select_repository(
    argument: Option<OsString>,
    environment: Option<OsString>,
    home: Option<OsString>,
) -> Result<PathBuf, String> {
    let path = argument.or(environment);
    match path {
        Some(path) if path.is_empty() => Err("repository path must not be empty".into()),
        Some(path) => {
            let path = PathBuf::from(path);
            if let Ok(relative) = path.strip_prefix("~") {
                Ok(
                    PathBuf::from(home.ok_or("HOME is required to expand ~/ in repository paths")?)
                        .join(relative),
                )
            } else {
                Ok(path)
            }
        }
        None => Ok(PathBuf::from(
            home.ok_or("HOME is required to locate ~/.dotfiles; use --repo")?,
        )
        .join(".dotfiles")),
    }
}

fn io_error(path: &Path, error: std::io::Error) -> String {
    format!("cannot load {}: {error}", path.display())
}

fn contained_path(path: &Path, repository: &Path) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(path).map_err(|error| io_error(path, error))?;
    if !resolved.starts_with(repository) {
        return Err(format!(
            "{} resolves outside repository {}",
            path.display(),
            repository.display()
        ));
    }
    Ok(resolved)
}

/// Discover direct `recipes/<name>.yaml` files in name order and validate their contents.
pub fn load_recipes(repository: &Path) -> Result<Vec<(String, Recipe)>, String> {
    let root = fs::canonicalize(repository).map_err(|error| io_error(repository, error))?;
    if !root.is_dir() {
        return Err(format!(
            "repository {} is not a directory",
            repository.display()
        ));
    }
    let directory = root.join("recipes");
    match fs::symlink_metadata(&directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(io_error(&directory, error)),
        Ok(_) => {}
    }
    let directory = contained_path(&directory, &root)?;
    let entries = fs::read_dir(&directory).map_err(|error| io_error(&directory, error))?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| io_error(&directory, error))?.path();
        if path.extension().is_none_or(|extension| extension != "yaml") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|name| {
                !name.is_empty()
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            })
            .ok_or_else(|| format!("invalid recipe name in {}", path.display()))?;
        paths.push((name.to_owned(), path));
    }
    paths.sort_by(|a, b| a.0.cmp(&b.0));
    paths
        .into_iter()
        .map(|(name, path)| {
            let resolved = contained_path(&path, &root)?;
            if !fs::metadata(&resolved)
                .map_err(|error| io_error(&path, error))?
                .is_file()
            {
                return Err(format!("recipe {} is not a regular file", path.display()));
            }
            let yaml = fs::read_to_string(&resolved).map_err(|error| io_error(&path, error))?;
            let recipe = parse_recipe(&yaml)
                .map_err(|error| format!("invalid recipe {}: {error}", path.display()))?;
            Ok((name, recipe))
        })
        .collect()
}
