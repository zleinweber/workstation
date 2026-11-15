use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

struct Fixture {
    root: TempDir,
    repo: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir_in(fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let repo = root.path().join("dot files");
        let home = root.path().join("fake home");
        fs::create_dir_all(repo.join("recipes")).unwrap();
        fs::create_dir_all(repo.join("bundles/shell")).unwrap();
        fs::create_dir(&home).unwrap();
        fs::write(repo.join("bundles/shell/.rc"), "source contents").unwrap();
        fs::write(home.join(".rc"), "preserve existing target").unwrap();
        Self { root, repo, home }
    }

    fn recipe(&self, yaml: &str) {
        fs::write(self.repo.join("recipes/test.yaml"), yaml).unwrap();
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_wstn"));
        command
            .args(["describe", "test", "--repo", "dot files"])
            .current_dir(self.root.path())
            .env("HOME", &self.home)
            .env_remove("WSTN_REPO")
            .env_remove("WSTN_RECIPE");
        command
    }

    fn run(&self, command: &mut Command) -> Output {
        let before = snapshot(self.root.path());
        let output = command.output().unwrap();
        assert_eq!(snapshot(self.root.path()), before, "describe changed files");
        output
    }

    fn success(&self) -> String {
        let output = self.run(&mut self.command());
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stderr.is_empty());
        String::from_utf8(output.stdout).unwrap()
    }

    fn failure(&self, command: &mut Command, expected: &str) {
        let output = self.run(command);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("test.yaml"), "{error}");
        assert!(error.contains("broken"), "{error}");
    }
}

// Record entries and contents, including symlinks without following them.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, (fs::FileType, Vec<u8>)> {
    let mut entries = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let kind = fs::symlink_metadata(&path).unwrap().file_type();
        let contents = if kind.is_symlink() {
            fs::read_link(&path)
                .unwrap()
                .into_os_string()
                .into_encoded_bytes()
        } else if kind.is_file() {
            fs::read(&path).unwrap()
        } else {
            entries.extend(snapshot(&path));
            Vec::new()
        };
        entries.insert(path, (kind, contents));
    }
    entries
}

fn links(source: &str, target: &str) -> String {
    format!(
        "version: 1\ndescription: Must not print on failure\nactions:\n\
         - {{id: valid, type: link, source: bundles/shell, target: /unused}}\n\
         - {{id: broken, type: link, source: '{source}', target: '{target}'}}\n"
    )
}

#[test]
fn describes_ordered_links_with_resolved_paths_and_policies_without_writes() {
    let fixture = Fixture::new();
    fixture.recipe(&format!(
        "version: 1\ndescription: Shell configuration\nactions:\n\
         - {{id: z-first, type: link, source: bundles/shell, target: '~/'}}\n\
         - {{id: a-second, type: link, source: bundles/shell, target: '{}', conflict: backup}}\n",
        fixture.home.join("not created").display()
    ));
    let source = fixture.repo.join("bundles/shell");
    assert_eq!(
        fixture.success(),
        format!(
            "Shell configuration\nz-first (link)\n  source: {}\n  target: {}\n  conflict: error\n\
             a-second (link)\n  source: {}\n  target: {}\n  conflict: backup\n",
            source.display(),
            fixture.home.join("").display(),
            source.display(),
            fixture.home.join("not created").display()
        )
    );
}

#[test]
fn empty_recipes_with_and_without_descriptions() {
    let fixture = Fixture::new();
    for (description, expected) in [
        ("", "No actions.\n"),
        (
            "description: Nothing to do\n",
            "Nothing to do\nNo actions.\n",
        ),
    ] {
        fixture.recipe(&format!("version: 1\n{description}actions: []\n"));
        assert_eq!(fixture.success(), expected);
    }
}

#[test]
fn rejects_bad_paths_before_printing_any_description() {
    let fixture = Fixture::new();
    for (source, target, expected) in [
        ("missing", "~/", "cannot access source path"),
        ("../fake home", "~/", "source path escapes repository"),
        ("/absolute", "~/", "source must be repository-relative"),
        ("bundles/shell", "relative", "target must be absolute"),
        ("bundles/shell", "~", "target must be absolute"),
        ("bundles/shell", "$HOME/config", "target must be absolute"),
        ("bundles/shell", "~other/config", "target must be absolute"),
    ] {
        fixture.recipe(&links(source, target));
        fixture.failure(&mut fixture.command(), expected);
    }
    fixture.recipe(&links("bundles/shell", "~/config"));
    fixture.failure(fixture.command().env_remove("HOME"), "HOME is required");
    fixture.failure(
        fixture.command().env("HOME", "relative"),
        "target must be absolute",
    );
}

#[test]
fn paths_do_not_expand_shell_expressions() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.repo.join("$HOME")).unwrap();
    fixture.recipe(&links(
        "$HOME",
        &format!("{}/$HOME/$(touch marker)", fixture.home.display()),
    ));
    let output = fixture.success();
    assert!(output.contains(&format!("source: {}", fixture.repo.join("$HOME").display())));
    assert!(output.contains(&format!(
        "target: {}/$HOME/$(touch marker)",
        fixture.home.display()
    )));
}

#[cfg(unix)]
#[test]
fn resolves_internal_source_symlinks_and_rejects_escapes() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let alias = fixture.repo.join("alias");
    symlink("bundles", &alias).unwrap();
    fixture.recipe(&links("alias/shell", "~/"));
    assert!(fixture.success().contains(&format!(
        "source: {}",
        fixture.repo.join("bundles/shell").display()
    )));

    fs::remove_file(&alias).unwrap();
    symlink(&fixture.home, &alias).unwrap();
    fixture.recipe(&links("alias", "~/"));
    fixture.failure(&mut fixture.command(), "source path escapes repository");
    fixture.recipe(&links("alias/.rc", "~/"));
    fixture.failure(&mut fixture.command(), "source path escapes repository");
}
