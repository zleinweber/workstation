//! Shared repository resolution, recipe discovery, and validated loading.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{Recipe, parse_recipe};

#[derive(Debug)]
pub struct LoadedRecipe {
    pub repository: PathBuf,
    pub path: PathBuf,
    pub recipe: Recipe,
}

pub fn resolve_repository(
    input: Option<&Path>,
    cwd: &Path,
    home: Option<&Path>,
) -> Result<PathBuf, String> {
    let input = input.unwrap_or_else(|| Path::new("~/.dotfiles"));
    if input.as_os_str().is_empty() {
        return Err("repository path must not be empty".into());
    }
    let expanded = match input.strip_prefix("~/") {
        Ok(relative) => home
            .ok_or("HOME is required to expand ~/ in the repository path")?
            .join(relative),
        Err(_) => input.to_path_buf(),
    };
    let path = cwd.join(expanded);
    let repository = fs::canonicalize(&path)
        .map_err(|e| format!("cannot access repository {}: {e}", path.display()))?;
    if !repository.is_dir() {
        return Err(format!("repository is not a directory: {}", path.display()));
    }
    Ok(repository)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
}

fn contained_path(repository: &Path, path: &Path) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(path)
        .map_err(|e| format!("cannot access recipe path {}: {e}", path.display()))?;
    if !resolved.starts_with(repository) {
        return Err(format!(
            "recipe path escapes repository: {}",
            path.display()
        ));
    }
    Ok(resolved)
}

/// List names only; recipe contents are validated when selected.
/// `repository` must be the canonical directory returned by `resolve_repository`.
pub fn discover_recipes(repository: &Path) -> Result<Vec<String>, String> {
    let directory = contained_path(repository, &repository.join("recipes"))?;
    let entries = fs::read_dir(&directory)
        .map_err(|e| format!("cannot read recipes directory {}: {e}", directory.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read recipe entry: {e}"))?;
        let filename = entry.file_name();
        let Some(name) = filename.to_str().and_then(|s| s.strip_suffix(".yaml")) else {
            continue;
        };
        if valid_name(name) && entry.path().is_file() {
            contained_path(repository, &entry.path())?;
            names.push(name.to_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// Load through the content parser without planning or executing actions.
/// `repository` must be the canonical directory returned by `resolve_repository`.
pub fn load_recipe(repository: PathBuf, name: Option<&str>) -> Result<LoadedRecipe, String> {
    let Some(name) = name else {
        let names = discover_recipes(&repository)?;
        let available = if names.is_empty() {
            "(none)".into()
        } else {
            names.join(", ")
        };
        return Err(format!(
            "no recipe selected; use wstn describe <recipe> or set WSTN_RECIPE.\nAvailable recipes: {available}"
        ));
    };
    if !valid_name(name) {
        return Err("invalid recipe name; expected [A-Za-z0-9_-]+ without an extension".into());
    }
    let path = contained_path(
        &repository,
        &repository.join("recipes").join(format!("{name}.yaml")),
    )?;
    let yaml = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read recipe {}: {e}", path.display()))?;
    let recipe =
        parse_recipe(&yaml).map_err(|e| format!("invalid recipe {}: {e}", path.display()))?;
    Ok(LoadedRecipe {
        repository,
        path,
        recipe,
    })
}
