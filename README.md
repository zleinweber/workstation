# Workstation

A recipe-driven dotfiles manager with basic configuration management, written in Rust.

**Status: initial implementation.** Help and version output work. An internal
V1 YAML parser validates empty recipes and link action fields; see
[example recipes](examples/recipes/). CLI recipe loading, linking, downloads,
and script execution are not implemented yet.

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
The parser uses Serde and `serde_yaml_ng`.

### Codespaces and devcontainers

On GitHub, select the branch you want to develop, then choose **Code → Codespaces
→ Create codespace**. For this initial PR, select `initialization`; after it is
merged, use `main`. The [devcontainer configuration](.devcontainer/devcontainer.json)
also works locally with Docker and VS Code's Dev Containers extension: choose
**Dev Containers: Reopen in Container**.

The container uses Ubuntu 26.04 LTS (Resolute) with APT, plus the standard Rust
and SSH server devcontainer features. It supplies Rust and native build tools,
runs as the non-root `vscode` user, and includes Rust Analyzer and EditorConfig
editor support. The Rust feature also supplies CodeLLDB and Even Better TOML
extensions, along with the `rust-analyzer` server and Rust sources. On creation,
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

### SSH and terminal editors

The devcontainer includes the standard SSH server feature so you can work in a
terminal editor through the GitHub CLI:

```sh
gh codespace ssh --codespace <codespace-name>
```

GitHub handles the SSH connection setup; this workflow does not require a public
SSH port or a manually configured password. It also supports workflows based on
`gh codespace ssh --config` and an SSH client such as `kitten ssh`.

For an existing Codespace, pull the updated branch and run **Codespaces: Rebuild
Container** from the VS Code command palette to install the SSH feature. Refresh
any locally generated Codespaces SSH configuration after rebuilding.

Neovim and personal editor configuration are not installed by this project;
install them separately or use your Codespaces dotfiles setup. For Rust LSP
support in Neovim, configure your LSP client to use the `rust-analyzer` executable
installed by the Rust feature. If it is missing after changing toolchains, run
`rustup component add rust-analyzer`.

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
