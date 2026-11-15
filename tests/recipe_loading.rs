use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::{TempDir, tempdir};
use workstation::recipe::{discover_recipes, load_recipe, resolve_repository};

const VALID: &str = "version: 1\nactions: []\n";

fn repository() -> TempDir {
    // macOS temporary paths can contain a /var -> /private/var symlink.
    let repo = tempfile::tempdir_in(fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    fs::create_dir(repo.path().join("recipes")).unwrap();
    repo
}

fn write_recipe(repo: &Path, name: &str, yaml: &str) {
    fs::write(repo.join("recipes").join(format!("{name}.yaml")), yaml).unwrap();
}

fn assert_error<T: Debug>(result: Result<T, String>, expected: &str) {
    let error = result.unwrap_err();
    assert!(
        error.contains(expected),
        "expected {expected:?} in {error:?}"
    );
}

fn assert_load_error(repo: &Path, name: &str, expected: &str) {
    assert_error(load_recipe(repo.to_path_buf(), Some(name)), expected);
}

fn home_repository(root: &Path) -> PathBuf {
    let home = root.join("fake home");
    let repo = home.join(".dotfiles");
    fs::create_dir_all(repo.join("recipes")).unwrap();
    write_recipe(&repo, "valid", VALID);
    home
}

fn command(cwd: &Path, home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wstn"));
    command
        .current_dir(cwd)
        .env("HOME", home)
        .env_remove("WSTN_REPO")
        .env_remove("WSTN_RECIPE");
    command
}

fn assert_cli_error(command: &mut Command, expected: &str) {
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
}

fn assert_empty_description(command: &mut Command) {
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, b"No actions.\n");
}

#[test]
fn repository_paths_and_home_expansion() {
    let root = tempdir().unwrap();
    let home = home_repository(root.path());
    let repo = home.join(".dotfiles");
    let expected = fs::canonicalize(&repo).unwrap();
    for path in [
        None,
        Some(Path::new("~/.dotfiles")),
        Some(Path::new("fake home/.dotfiles")),
        Some(repo.as_path()),
    ] {
        assert_eq!(
            resolve_repository(path, root.path(), Some(&home)).unwrap(),
            expected
        );
    }
    fs::create_dir(root.path().join("$HOME")).unwrap();
    assert_eq!(
        resolve_repository(Some(Path::new("$HOME")), root.path(), Some(&home)).unwrap(),
        fs::canonicalize(root.path().join("$HOME")).unwrap()
    );
}

#[test]
fn rejects_invalid_repository_inputs() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("file"), "").unwrap();
    for (path, expected) in [
        (None, "HOME"),
        (Some(""), "must not be empty"),
        (Some("missing"), "cannot access repository"),
        (Some("file"), "not a directory"),
    ] {
        assert_error(
            resolve_repository(path.map(Path::new), root.path(), None),
            expected,
        );
    }
}

#[test]
fn discovers_only_yaml_names_in_sorted_order() {
    let repo = repository();
    for name in ["z-last", "Alpha_2", "first", "bad.name", "has space"] {
        write_recipe(repo.path(), name, VALID);
    }
    fs::write(repo.path().join("recipes/other.yml"), VALID).unwrap();
    fs::create_dir(repo.path().join("recipes/directory.yaml")).unwrap();
    assert_eq!(
        discover_recipes(repo.path()).unwrap(),
        ["Alpha_2", "first", "z-last"]
    );
}

#[test]
fn missing_selection_lists_available_recipes() {
    let repo = repository();
    assert_error(
        load_recipe(repo.path().to_path_buf(), None),
        "Available recipes: (none)",
    );
    for name in ["z-last", "Alpha_2"] {
        write_recipe(repo.path(), name, VALID);
    }
    let error = load_recipe(repo.path().to_path_buf(), None).unwrap_err();
    assert!(error.contains("wstn describe <recipe> or set WSTN_RECIPE"));
    assert!(error.contains("Available recipes: Alpha_2, z-last"));
}

#[test]
fn loads_valid_recipe() {
    let repo = repository();
    write_recipe(repo.path(), "valid", VALID);
    let loaded = load_recipe(repo.path().to_path_buf(), Some("valid")).unwrap();
    assert_eq!(loaded.repository, repo.path());
    assert_eq!(loaded.path, repo.path().join("recipes/valid.yaml"));
    assert_eq!(loaded.recipe.version, 1);
    assert!(loaded.recipe.actions.is_empty());
}

#[test]
fn rejects_invalid_recipe_names() {
    let repo = repository();
    for name in [
        "",
        "../valid",
        "valid.yaml",
        "valid.yml",
        "/valid",
        "a/b",
        "a\\b",
        "has space",
        "é",
    ] {
        assert_load_error(repo.path(), name, "invalid recipe name");
    }
}

#[test]
fn reports_recipe_parser_errors() {
    let repo = repository();
    // Parser-specific validation cases live in tests/recipes.rs.
    write_recipe(repo.path(), "invalid", "version: [");
    assert_load_error(repo.path(), "invalid", "invalid recipe");
}

#[test]
fn reports_recipe_file_errors() {
    let repo = repository();
    assert_load_error(repo.path(), "missing", "cannot access recipe path");
    fs::write(repo.path().join("recipes/binary.yaml"), [0xff]).unwrap();
    fs::create_dir(repo.path().join("recipes/directory.yaml")).unwrap();
    for name in ["binary", "directory"] {
        assert_load_error(repo.path(), name, "cannot read recipe");
    }
}

#[test]
fn reports_missing_recipes_directory() {
    let repo = repository();
    fs::remove_dir(repo.path().join("recipes")).unwrap();
    assert_error(discover_recipes(repo.path()), "cannot access recipe path");
}

#[test]
fn cli_inputs_override_environment() {
    let root = tempdir().unwrap();
    let home = home_repository(root.path());
    assert_empty_description(
        command(root.path(), &home)
            .env("WSTN_REPO", "missing")
            .env("WSTN_RECIPE", "invalid.yaml")
            .args(["describe", "valid", "--repo", "fake home/.dotfiles"]),
    );
}

#[test]
fn environment_inputs_override_defaults() {
    let root = tempdir().unwrap();
    home_repository(root.path());
    assert_empty_description(
        command(root.path(), root.path())
            .env("WSTN_REPO", "~/fake home/.dotfiles")
            .env("WSTN_RECIPE", "valid")
            .arg("describe"),
    );
}

#[test]
fn default_repository_and_selection_diagnostics() {
    let root = tempdir().unwrap();
    let home = home_repository(root.path());
    assert_cli_error(
        command(root.path(), &home).arg("describe"),
        "Available recipes: valid",
    );
    assert_empty_description(command(root.path(), &home).args(["describe", "valid"]));
    assert_eq!(
        fs::read_to_string(home.join(".dotfiles/recipes/valid.yaml")).unwrap(),
        VALID
    );
    assert_eq!(fs::read_dir(&home).unwrap().count(), 1);
}

#[test]
fn cli_rejects_missing_option_value() {
    let root = tempdir().unwrap();
    assert_cli_error(
        command(root.path(), root.path()).args(["describe", "--repo"]),
        "--repo",
    );
}

#[test]
fn help_and_version_succeed() {
    let root = tempdir().unwrap();
    for option in ["--help", "--version"] {
        assert!(
            command(root.path(), root.path())
                .arg(option)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}

#[cfg(unix)]
#[test]
fn recipe_symlinks_must_stay_inside_repository() {
    use std::os::unix::fs::symlink;
    let repo = repository();
    let outside = repository();
    write_recipe(outside.path(), "external", VALID);
    write_recipe(repo.path(), "local", VALID);
    symlink("local.yaml", repo.path().join("recipes/alias.yaml")).unwrap();
    load_recipe(repo.path().to_path_buf(), Some("alias")).unwrap();
    symlink(
        outside.path().join("recipes/external.yaml"),
        repo.path().join("recipes/escape.yaml"),
    )
    .unwrap();
    assert_load_error(repo.path(), "escape", "escapes repository");
    assert_error(discover_recipes(repo.path()), "escapes repository");
}

#[cfg(unix)]
#[test]
fn recipes_directory_symlink_must_stay_inside_repository() {
    use std::os::unix::fs::symlink;
    let repo = repository();
    let outside = repository();
    write_recipe(outside.path(), "external", VALID);
    fs::remove_dir(repo.path().join("recipes")).unwrap();
    symlink(outside.path().join("recipes"), repo.path().join("recipes")).unwrap();
    assert_load_error(repo.path(), "external", "escapes repository");
    assert_error(discover_recipes(repo.path()), "escapes repository");
}

#[cfg(unix)]
#[test]
fn reports_unreadable_recipe() {
    use std::os::unix::fs::PermissionsExt;
    let repo = repository();
    write_recipe(repo.path(), "private", VALID);
    let path = repo.path().join("recipes/private.yaml");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let readable = fs::read(&path).is_ok();
    let result = load_recipe(repo.path().to_path_buf(), Some("private"));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    if readable {
        eprintln!(
            "skipping permission-denied assertion: this user can read mode-000 files (e.g. root)"
        );
        return;
    }
    assert_error(result, "cannot read recipe");
}

#[test]
fn cli_parsing_does_not_access_repository() {
    use clap::Parser;
    use workstation::cli::{Cli, Command};

    let cli = Cli::try_parse_from([
        "wstn",
        "describe",
        "chosen",
        "--repo",
        "/nonexistent/path with spaces",
    ])
    .unwrap();
    let Some(Command::Describe(inputs)) = cli.command else {
        panic!("expected describe inputs");
    };
    assert_eq!(inputs.recipe.as_deref(), Some("chosen"));
    assert_eq!(
        inputs.repo.as_deref(),
        Some(Path::new("/nonexistent/path with spaces"))
    );
}
