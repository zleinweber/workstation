# Workstation

A recipe-driven dotfiles manager with basic configuration management, written in Rust.

**Status: project scaffold.** Help and version output work. Recipes, linking,
downloads, and script execution are proposed and are not implemented yet.

Workstation will take a local dotfiles repository and a named YAML recipe, explain
the recipe, and apply its actions. Configuration files and personal scripts stay
in the user's repository; the executable supplies the reusable framework.

## Intended experience

The following commands describe the planned interface, not current functionality:

```sh
wstn describe macos --repo ~/.dotfiles
wstn setup macos --repo ~/.dotfiles
wstn setup codespaces --dry-run
```

The initial action set is package-based symlinks, verified file downloads, and
custom scripts. A small `setup` wrapper in a user's repository will preserve
GitHub Codespaces integration. Prebuilt releases are planned so end users will
not need a Rust toolchain.

## Development

Install Rust through [rustup](https://rustup.rs/), then run:

```sh
cargo run -- --help
cargo run -- --version
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

The project uses Rust 2024, with Rust 1.85 as the minimum supported version.
`rust-toolchain.toml` selects stable Rust with rustfmt and Clippy. CI checks stable
and the minimum version on Linux and macOS. Cargo.lock is committed for the CLI.
There are no runtime dependencies at the scaffold stage.

## Project documents

- [Design](docs/design.md): scope, configuration, action semantics, architecture,
  migration, and open decisions.
- [Implementation plan](docs/plan.md): milestones and acceptance checks.
- [Contributing](CONTRIBUTING.md): local development and contribution expectations.

## Name

The project and Cargo package are named **Workstation** (`workstation`); the
command is **`wstn`**. The name reflects its purpose: setting up a personal working
environment from version-controlled configuration.

## License

[MIT](LICENSE).
