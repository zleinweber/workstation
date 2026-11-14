use std::fs;
use std::process::{Command, Output};

use tempfile::{TempDir, tempdir};

const VALID: &str = "version: 1\nactions: []\n";
const SINGLE_RECIPE_TABLE: &str = "Name  Description\ntest  \n";

struct Fixture(TempDir);

impl Fixture {
    fn new() -> Self {
        let root = tempdir().unwrap();
        fs::create_dir(root.path().join("recipes")).unwrap();
        fs::create_dir(root.path().join("home")).unwrap();
        Self(root)
    }

    fn recipe(&self, name: &str, content: &str) {
        fs::write(
            self.0.path().join("recipes").join(format!("{name}.yaml")),
            content,
        )
        .unwrap();
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_wstn"));
        command
            .arg("list")
            .current_dir(self.0.path())
            .env_remove("WSTN_REPO")
            .env_remove("WSTN_RECIPE")
            .env("HOME", self.0.path().join("home"));
        command
    }
}

fn stdout(command: &mut Command) -> String {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

fn failure(command: &mut Command) -> String {
    let Output {
        status,
        stdout,
        stderr,
    } = command.output().unwrap();
    let error = String::from_utf8(stderr).unwrap();
    assert_eq!(status.code(), Some(2), "{error}");
    assert!(stdout.is_empty(), "unexpected stdout: {stdout:?}");
    error
}

#[test]
fn table_is_sorted_with_blank_omitted_descriptions_and_ignores_recipe_selection() {
    let fixture = Fixture::new();
    fixture.recipe("z-last", VALID);
    fixture.recipe(
        "Alpha",
        "version: 1\ndescription: First recipe\nactions: []\n",
    );

    let output = stdout(
        fixture
            .command()
            .args(["--repo", "."])
            .env("WSTN_RECIPE", "z-last"),
    );
    assert_eq!(
        output,
        "Name    Description\nAlpha   First recipe\nz-last  \n"
    );
}

#[test]
fn table_preserves_header_width_and_keeps_descriptions_on_one_row() {
    let fixture = Fixture::new();
    fixture.recipe(
        "a",
        r#"version: 1
description: "first\nsecond\rthird\tfourth"
actions: []
"#,
    );

    assert_eq!(
        stdout(fixture.command().args(["--repo", "."])),
        "Name  Description\na     first second third fourth\n",
    );
}

#[test]
fn repository_argument_overrides_environment() {
    let fixture = Fixture::new();
    fixture.recipe("test", VALID);
    assert_eq!(
        stdout(
            fixture
                .command()
                .args(["--repo", "."])
                .env("WSTN_REPO", "missing")
        ),
        SINGLE_RECIPE_TABLE,
    );
}

#[test]
fn repository_environment_overrides_default() {
    let fixture = Fixture::new();
    fixture.recipe("test", VALID);
    assert_eq!(
        stdout(fixture.command().env("WSTN_REPO", ".")),
        SINGLE_RECIPE_TABLE,
    );
}

#[test]
fn repository_defaults_to_dotfiles_in_home() {
    let fixture = Fixture::new();
    let recipes = fixture.0.path().join("home/.dotfiles/recipes");
    fs::create_dir_all(&recipes).unwrap();
    fs::write(recipes.join("test.yaml"), VALID).unwrap();

    assert_eq!(stdout(&mut fixture.command()), SINGLE_RECIPE_TABLE);
}

#[test]
fn empty_recipe_directory_is_clear() {
    let fixture = Fixture::new();
    assert!(stdout(fixture.command().args(["--repo", "."])).contains("No recipes available"));
}

#[test]
fn missing_recipes_directory_reports_discovery_failure() {
    let fixture = Fixture::new();
    fs::remove_dir(fixture.0.path().join("recipes")).unwrap();
    assert!(failure(fixture.command().args(["--repo", "."])).contains("cannot access recipe path"));
}

#[test]
fn missing_repository_reports_resolution_failure() {
    let fixture = Fixture::new();
    assert!(
        failure(fixture.command().args(["--repo", "missing"])).contains("cannot access repository")
    );
}

#[test]
fn invalid_arguments_are_rejected() {
    let fixture = Fixture::new();
    for argument in ["--repo", "extra", "--unknown"] {
        let error = failure(fixture.command().arg(argument));
        assert!(error.contains(argument), "argument {argument:?}: {error}");
    }
}

#[test]
fn invalid_recipe_fails_without_printing_a_partial_table() {
    let fixture = Fixture::new();
    fixture.recipe("a", VALID);
    fixture.recipe("broken", "version: 2\nactions: []");

    let error = failure(fixture.command().args(["--repo", "."]));
    assert!(error.contains("broken.yaml"), "{error}");
    assert!(error.contains("unsupported recipe version"), "{error}");
}

#[test]
fn help_does_not_load_repository() {
    let fixture = Fixture::new();
    assert!(
        stdout(fixture.command().arg("--help").env("WSTN_REPO", "missing")).contains("WSTN_REPO")
    );
}
