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

### Codespaces and devcontainers

On GitHub, select the branch you want to develop, then choose **Code → Codespaces
→ Create codespace**. For this initial PR, select `initialization`; after it is
merged, use `main`. The [devcontainer configuration](.devcontainer/devcontainer.json)
also works locally with Docker and VS Code's Dev Containers extension: choose
**Dev Containers: Reopen in Container**.

The container supplies Rust and native build tools, runs as the non-root `vscode`
user, and installs Rust Analyzer and EditorConfig editor support. On creation,
it updates stable Rust and builds `wstn` with the committed lockfile. The repository's
`rust-toolchain.toml` supplies rustfmt and Clippy. Rust files format on save, and
Rust Analyzer uses Clippy for diagnostics.

In the container terminal, run:

```sh
cargo run --locked -- --help
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

To also check the minimum supported compiler, as CI does:

```sh
rustup toolchain install 1.85.0 --profile minimal
cargo +1.85.0 build --locked
cargo +1.85.0 test --locked --all-targets
```

The container is a Linux development environment; macOS coverage remains in CI.
It does not apply a dotfiles recipe or configure the host machine.

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
