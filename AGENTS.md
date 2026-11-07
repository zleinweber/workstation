# Repository guidance

## Start here

- Read `CONTRIBUTING.md` before changing the project.
- Read `docs/design.md` and `docs/plan.md` before implementing features.
- Treat design examples and milestones as proposals, not implemented behavior.
- Check the code before documenting a command as supported.
- Keep README examples and design documentation aligned with behavior changes.
- Discuss recipe semantics before introducing compatibility commitments.

## Project boundaries

- Use Workstation for the project name, `workstation` for the Cargo package,
  and `wstn` for the executable; environment overrides use `WSTN_`.
- Keep the framework generic: CLI, schema, actions, tests, and release tooling.
- Keep personal recipes, package lists, and application installers in user
  or example repositories rather than embedding them in Rust.
- Preserve Linux and macOS support when changing filesystem or process code.
- Consult the design's deferred scope before expanding the feature set.

## Rust and dependencies

- Use the stable toolchain selected by `rust-toolchain.toml`.
- Keep Rust 2024 code compatible with the Rust 1.85 minimum in `Cargo.toml`.
- Preserve the application's prohibition on unsafe code.
- Add dependencies only to meet an implemented requirement.
- Commit `Cargo.lock` updates together with dependency changes.
- Follow `.editorconfig` for whitespace and file formatting.

## Validation

Run the relevant README checks before submitting changes; for Rust changes use:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked
cargo run --locked -- --help
cargo run --locked -- --version
```

- Check minimum-version compatibility for new APIs and dependency changes;
  CI builds and tests stable and Rust 1.85.0 on Linux and macOS.
- Add focused behavior tests for actions, including conflicts and interruptions.
- Use temporary repositories and target/home directories in filesystem tests.
- Never provision the contributor's machine or run personal dotfiles as a test.
- Use controlled download fixtures rather than live installer endpoints.

## Action implementation

- Keep resolution and validation shared across describe, dry-run, and setup.
- Keep previews free of recipe execution, downloads, and target writes.
- Validate recipe structure and repository sources before mutation.
- Preserve user files on conflicts and failed or interrupted operations.
- Never log environment values that may contain credentials.
