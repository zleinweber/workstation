use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "wstn list {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("recipes")).unwrap();
        Self(path)
    }
    fn recipe(&self, name: &str, content: &str) {
        fs::write(self.0.join("recipes").join(name), content).unwrap();
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_wstn"));
        command
            .current_dir(&self.0)
            .env_remove("WSTN_REPO")
            .env_remove("WSTN_RECIPE")
            .env("HOME", &self.0);
        command
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}
fn failure(output: Output) -> String {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    String::from_utf8(output.stderr).unwrap()
}

#[test]
fn sorted_table_validates_recipes_and_ignores_recipe_selection() {
    let fixture = Fixture::new();
    fixture.recipe("z-last.yaml", "version: 1\nactions: []\n");
    fixture.recipe(
        "Alpha.yaml",
        "version: 1\ndescription: First recipe\nactions: []\n",
    );
    fixture.recipe("ignored.yml", "invalid");
    fs::create_dir(fixture.0.join("recipes/nested")).unwrap();
    let output = stdout(
        fixture
            .command()
            .args(["list", "--repo", "."])
            .env("WSTN_REPO", "missing")
            .env("WSTN_RECIPE", "z-last")
            .output()
            .unwrap(),
    );
    assert_eq!(
        output,
        "Name    Description\nAlpha   First recipe\nz-last  \n"
    );
    assert!(!fixture.0.join(".dotfiles").exists());
}

#[test]
fn repository_environment_and_default_and_tilde_expansion() {
    let fixture = Fixture::new();
    fixture.recipe("test.yaml", "version: 1\nactions: []");
    let output = stdout(
        fixture
            .command()
            .arg("list")
            .env("WSTN_REPO", ".")
            .output()
            .unwrap(),
    );
    assert!(output.contains("test"));
    fs::rename(fixture.0.join("recipes"), fixture.0.join("saved")).unwrap();
    fs::create_dir(fixture.0.join(".dotfiles")).unwrap();
    fs::rename(fixture.0.join("saved"), fixture.0.join(".dotfiles/recipes")).unwrap();
    assert!(stdout(fixture.command().arg("list").output().unwrap()).contains("test"));
    assert!(
        stdout(
            fixture
                .command()
                .args(["list", "--repo", "~/.dotfiles"])
                .output()
                .unwrap()
        )
        .contains("test")
    );
}

#[test]
fn empty_recipe_directory_is_clear_and_missing_directory_fails() {
    let fixture = Fixture::new();
    assert!(
        stdout(
            fixture
                .command()
                .args(["list", "--repo", "."])
                .output()
                .unwrap()
        )
        .contains("No recipes available")
    );
    fs::remove_dir(fixture.0.join("recipes")).unwrap();
    assert!(
        failure(
            fixture
                .command()
                .args(["list", "--repo", "."])
                .output()
                .unwrap()
        )
        .contains("cannot access recipe path")
    );
}

#[test]
fn invalid_repositories_recipes_and_arguments_fail_without_partial_table() {
    let fixture = Fixture::new();
    for arguments in [
        vec!["list", "--repo", "missing"],
        vec!["list", "--repo"],
        vec!["list", "extra"],
        vec!["list", "--unknown"],
        vec!["setup"],
    ] {
        assert!(failure(fixture.command().args(arguments).output().unwrap()).contains("error:"));
    }
    fixture.recipe("a.yaml", "version: 1\nactions: []");
    fixture.recipe("broken.yaml", "version: 2\nactions: []");
    let error = failure(
        fixture
            .command()
            .args(["list", "--repo", "."])
            .output()
            .unwrap(),
    );
    assert!(error.contains("broken.yaml") && error.contains("unsupported recipe version"));
    fixture.recipe("broken.yaml", "version: 1\nactions: []");
    fixture.recipe("bad.name.yaml", "version: 1\nactions: []");
    assert!(
        !stdout(
            fixture
                .command()
                .args(["list", "--repo", "."])
                .output()
                .unwrap()
        )
        .contains("bad.name")
    );
}

#[cfg(unix)]
#[test]
fn rejects_recipe_and_directory_symlinks_escaping_repository() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let outside = Fixture::new();
    outside.recipe("external.yaml", "version: 1\nactions: []");
    symlink(
        outside.0.join("recipes/external.yaml"),
        fixture.0.join("recipes/escape.yaml"),
    )
    .unwrap();
    assert!(
        failure(
            fixture
                .command()
                .args(["list", "--repo", "."])
                .output()
                .unwrap()
        )
        .contains("escapes repository")
    );
    fs::remove_file(fixture.0.join("recipes/escape.yaml")).unwrap();
    fs::remove_dir(fixture.0.join("recipes")).unwrap();
    symlink(outside.0.join("recipes"), fixture.0.join("recipes")).unwrap();
    assert!(
        failure(
            fixture
                .command()
                .args(["list", "--repo", "."])
                .output()
                .unwrap()
        )
        .contains("escapes repository")
    );
}

#[test]
fn help_does_not_load_repository() {
    let fixture = Fixture::new();
    assert!(
        stdout(
            fixture
                .command()
                .args(["list", "--help"])
                .env("WSTN_REPO", "missing")
                .output()
                .unwrap()
        )
        .contains("WSTN_REPO")
    );
}
