use std::fs;
use std::path::Path;
use std::process::{Command, Output};

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

fn command(cwd: &Path, home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wstn"));
    command
        .current_dir(cwd)
        .env("HOME", home)
        .env_remove("WSTN_REPO")
        .env_remove("WSTN_RECIPE");
    command
}

fn failure(output: Output) -> String {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    String::from_utf8(output.stderr).unwrap()
}

#[test]
fn repository_paths_and_home_expansion() {
    let root = tempdir().unwrap();
    let home = root.path().join("fake home");
    let repo = home.join(".dotfiles");
    fs::create_dir_all(&repo).unwrap();
    let expected = fs::canonicalize(&repo).unwrap();
    for path in [
        None,
        Some(Path::new("~/.dotfiles")),
        Some(Path::new("fake home/.dotfiles")),
        Some(repo.as_path()),
    ] {
        // Only a leading ~/ is expanded; the repository need not be the cwd.
        assert_eq!(
            resolve_repository(path, root.path(), Some(&home)).unwrap(),
            expected
        );
    }
    assert!(
        resolve_repository(None, root.path(), None)
            .unwrap_err()
            .contains("HOME")
    );
    assert!(resolve_repository(Some(Path::new("")), root.path(), Some(&home)).is_err());
    assert!(
        resolve_repository(Some(Path::new("missing")), root.path(), None)
            .unwrap_err()
            .contains("cannot access repository")
    );
    fs::write(root.path().join("file"), "").unwrap();
    assert!(
        resolve_repository(Some(Path::new("file")), root.path(), None)
            .unwrap_err()
            .contains("not a directory")
    );
    fs::create_dir(root.path().join("$HOME")).unwrap();
    assert_eq!(
        resolve_repository(Some(Path::new("$HOME")), root.path(), Some(&home)).unwrap(),
        fs::canonicalize(root.path().join("$HOME")).unwrap()
    );
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
    let error = load_recipe(repo.path().to_path_buf(), None).unwrap_err();
    assert!(error.contains("wstn describe <recipe> or set WSTN_RECIPE"));
    assert!(error.contains("Available recipes: Alpha_2, first, z-last"));
    let empty = repository();
    assert!(
        load_recipe(empty.path().to_path_buf(), None)
            .unwrap_err()
            .contains("Available recipes: (none)")
    );
}

#[test]
fn loads_with_existing_parser_and_rejects_bad_inputs() {
    let repo = repository();
    write_recipe(repo.path(), "valid", VALID);
    let loaded = load_recipe(repo.path().to_path_buf(), Some("valid")).unwrap();
    assert_eq!(loaded.repository, repo.path());
    assert_eq!(loaded.path, repo.path().join("recipes/valid.yaml"));
    assert_eq!(loaded.recipe.version, 1);
    assert!(loaded.recipe.actions.is_empty());
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
        assert!(
            load_recipe(repo.path().to_path_buf(), Some(name))
                .unwrap_err()
                .contains("invalid recipe name")
        );
    }
    assert!(
        load_recipe(repo.path().to_path_buf(), Some("missing"))
            .unwrap_err()
            .contains("cannot access recipe path")
    );
    for yaml in [
        "version: [",
        "version: 2\nactions: []",
        "version: 1\nactions: []\nextra: true",
        "version: 1\nversion: 1\nactions: []",
    ] {
        write_recipe(repo.path(), "invalid", yaml);
        assert!(
            load_recipe(repo.path().to_path_buf(), Some("invalid"))
                .unwrap_err()
                .contains("invalid recipe")
        );
    }
    fs::write(repo.path().join("recipes/binary.yaml"), [0xff]).unwrap();
    assert!(
        load_recipe(repo.path().to_path_buf(), Some("binary"))
            .unwrap_err()
            .contains("cannot read recipe")
    );
    fs::create_dir(repo.path().join("recipes/directory.yaml")).unwrap();
    assert!(
        load_recipe(repo.path().to_path_buf(), Some("directory"))
            .unwrap_err()
            .contains("cannot read recipe")
    );
    fs::remove_dir_all(repo.path().join("recipes")).unwrap();
    assert!(
        discover_recipes(repo.path())
            .unwrap_err()
            .contains("cannot access recipe path")
    );
}

#[test]
fn cli_precedence_and_loading_from_outside_repository() {
    let root = tempdir().unwrap();
    let home = root.path().join("fake home");
    let repo = home.join(".dotfiles");
    fs::create_dir_all(repo.join("recipes")).unwrap();
    write_recipe(&repo, "valid", VALID);
    // CLI overrides both environment inputs, including a nonexistent repo.
    let output = command(root.path(), &home)
        .env("WSTN_REPO", "missing")
        .env("WSTN_RECIPE", "invalid.yaml")
        .args(["describe", "valid", "--repo", "fake home/.dotfiles"])
        .output()
        .unwrap();
    assert!(failure(output).contains("describe rendering is not implemented yet"));
    // Environment overrides the default repository and supplies the recipe.
    let output = command(root.path(), root.path())
        .env("WSTN_REPO", "~/fake home/.dotfiles")
        .env("WSTN_RECIPE", "valid")
        .arg("describe")
        .output()
        .unwrap();
    assert!(failure(output).contains("describe rendering is not implemented yet"));
    // Default repository and no recipe: discovery is an input error.
    let output = command(root.path(), &home)
        .arg("describe")
        .output()
        .unwrap();
    assert!(failure(output).contains("Available recipes: valid"));
    let output = command(root.path(), &home)
        .args(["describe", "valid"])
        .output()
        .unwrap();
    assert!(failure(output).contains("describe rendering is not implemented yet"));
    for args in [
        vec!["describe", "--repo"],
        vec!["describe", "one", "two"],
        vec!["describe", "--unknown"],
        vec!["setup"],
    ] {
        failure(command(root.path(), &home).args(args).output().unwrap());
    }
    for args in [
        vec![],
        vec!["--help"],
        vec!["--version"],
        vec!["describe", "--help"],
    ] {
        assert!(
            command(root.path(), &home)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert_eq!(
        fs::read_to_string(repo.join("recipes/valid.yaml")).unwrap(),
        VALID
    );
    assert_eq!(fs::read_dir(&home).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn symlinks_must_stay_inside_repository() {
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
    assert!(
        load_recipe(repo.path().to_path_buf(), Some("escape"))
            .unwrap_err()
            .contains("escapes repository")
    );
    assert!(
        discover_recipes(repo.path())
            .unwrap_err()
            .contains("escapes repository")
    );
    fs::remove_dir_all(repo.path().join("recipes")).unwrap();
    symlink(outside.path().join("recipes"), repo.path().join("recipes")).unwrap();
    assert!(
        load_recipe(repo.path().to_path_buf(), Some("external"))
            .unwrap_err()
            .contains("escapes repository")
    );
    assert!(
        discover_recipes(repo.path())
            .unwrap_err()
            .contains("escapes repository")
    );
}

#[cfg(unix)]
#[test]
fn unreadable_repository_and_recipe_report_io_errors() {
    use std::os::unix::fs::PermissionsExt;
    let repo = repository();
    write_recipe(repo.path(), "private", VALID);
    let path = repo.path().join("recipes/private.yaml");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let error = load_recipe(repo.path().to_path_buf(), Some("private")).unwrap_err();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(error.contains("cannot read recipe"));
    fs::set_permissions(repo.path(), fs::Permissions::from_mode(0o000)).unwrap();
    let error = discover_recipes(repo.path()).unwrap_err();
    fs::set_permissions(repo.path(), fs::Permissions::from_mode(0o700)).unwrap();
    assert!(error.contains("cannot access recipe path"));
}

#[test]
fn cli_parsing_does_not_access_repository() {
    use clap::Parser;
    use workstation::cli::{Cli, Command};

    for args in [
        vec![
            "wstn",
            "describe",
            "chosen",
            "--repo",
            "/nonexistent/path with spaces",
        ],
        vec![
            "wstn",
            "describe",
            "--repo",
            "/nonexistent/path with spaces",
            "chosen",
        ],
    ] {
        let cli = Cli::try_parse_from(args).unwrap();
        let Some(Command::Describe(inputs)) = cli.command else {
            panic!("expected describe inputs");
        };
        assert_eq!(inputs.recipe.as_deref(), Some("chosen"));
        assert_eq!(
            inputs.repo.as_deref(),
            Some(Path::new("/nonexistent/path with spaces"))
        );
    }
}
