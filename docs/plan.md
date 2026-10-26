# Workstation implementation plan

Build a usable vertical slice before broadening the action set. The initial
deliverable is a Rust scaffold plus a reviewable design, not a working installer.
Each milestone below has an observable completion condition.

## 0 Project foundation

- Name selected: Workstation, Cargo package `workstation`, executable `wstn`.
- Create a separate Git repository with a Rust 2024 binary crate and lockfile.
- Add README, MIT license, contribution instructions, and the design and plan.
- Configure rustfmt, Clippy, and Linux/macOS CI with an explicit minimum Rust version.
- Keep unsupported commands honest: no placeholder setup command that reports success.

Done when formatting, linting, compilation, and help/version smoke checks pass.
Publication and release binaries are separate from local scaffolding.

## 1 Recipes and describe

- Add Clap and choose a maintained YAML parser after checking duplicate-key behavior.
- Implement repository/recipe selection and CLI-over-environment precedence.
- Implement version 1 types, strict validation, and editor JSON Schema.
- Implement `describe` with ordered actions and resolved paths, without mutation.
- Add an example recipe and temporary-repository integration fixtures.

Verify malformed YAML, unknown fields, unsupported versions, duplicate IDs/keys,
missing recipes, invalid recipe names, path escapes, and paths containing spaces.
Run describe from outside the repository and assert that the fake home is unchanged.

Done when one small recipe from the personal repository can be represented and
explained without invoking its scripts or touching the target machine.

## 2 Link and preview

- Introduce a shared plan representation and local filesystem inspection.
- Implement file symlinks, real parent directories, and deterministic traversal.
- Implement error and unique-backup conflict policies.
- Add `setup --dry-run`, sequential execution, and action summaries.
- Add a per-user setup lock before exposing mutation as a supported workflow.

Verify an empty target, unchanged second run, broken links, foreign links, file
and directory conflicts, overlapping packages, ancestor symlinks, unsupported
source entries, failed backups, and concurrent setup attempts. Assert dry-run
makes no changes and each backup preserves the original contents.

Done when a shell/editor configuration installs correctly and a second run reports
unchanged, on both Linux and macOS.

## 3 Scripts and a real migration slice

- Add direct argv execution, repository-relative cwd, environment overrides,
  declarative platform conditions, and the optional creates guard.
- Stream output, stop on failure, and handle interruption and child cleanup.
- Show scripts as opaque actions in previews without running them.
- Translate one existing recipe using personal scripts for application installs.

Verify argument boundaries, cwd, conditions, guards, failure summaries, interrupted
children, and an unexecuted action after failure. Run the migration in a disposable
Codespaces-like Linux environment and a disposable macOS environment when available.

Done when the migrated slice performs useful setup while the Rust code contains
no knowledge of specific personal applications.

## 4 Verified downloads

- Add HTTPS downloads, required checksums, timeouts, and bounded redirects.
- Add atomic replacement, executable permissions, and explicit replacement policy.
- Include download inspection in dry-run and cleanup of temporary files on failure.

Verify checksum hits and mismatches, corrupt/partial responses, redirects, timeouts,
existing foreign files, symlink targets, executable mode changes, and failed writes.
Use a controlled test server with test certificates trusted only by the test client;
never disable TLS verification in the production implementation.

Done when a pinned direct binary download can replace an equivalent bespoke script
without leaving a partial binary or silently overwriting a conflicting file.

## 5 First release and bootstrap

- Build Linux/macOS x86_64/aarch64 binaries and publish checksums.
- Document platform requirements and validate artifacts on actual target systems
  or record any architecture that was only cross-compiled.
- Add a pinned-version bootstrap example and the thin Codespaces setup wrapper.
- Add installation, upgrade, failure recovery, and recipe migration documentation.
- Finish one full personal recipe, then try the conventions with a second example
  repository to expose assumptions tied to the original dotfiles.

Done when a clean supported machine can install and run the tool without a Rust,
Python, or Stow installation, and the documented Codespaces entry point works.
User scripts' separate dependencies must be explicit.

## Later work

Prioritize from real use: ownership tracking and unlink, recipe composition,
archives, package-manager adapters, templates, and structured machine-readable
output. Avoid adding these solely for parity with established dotfiles managers.

## Development checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked
cargo run --locked -- --help
cargo run --locked -- --version
```

Feature tests must use isolated fixtures. Real provisioning tests are explicit,
disposable-environment checks, never a developer's normal home directory.
