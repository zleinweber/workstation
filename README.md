# Workstation

A recipe-driven dotfiles manager with basic configuration management, written in Rust.

**Status: initial implementation.** Help and version output work. An internal
V1 YAML parser validates empty recipes and link action fields; see
[example recipes](examples/recipes/). `describe` loads and validates a selected
recipe, but description rendering, linking, downloads, and script execution are
not implemented yet. Until rendering is available, even a valid recipe returns
exit code 2 with an explicit diagnostic.

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

## Recipe selection

`wstn describe [recipe] --repo <path>` accepts repository and recipe inputs.
`--repo` overrides `WSTN_REPO`, defaulting to `~/.dotfiles`; the positional
recipe overrides `WSTN_RECIPE`. Relative repository paths use the invocation
directory, and a leading `~/` expands using `HOME`. No other shell expansion
is performed. Names use `[A-Za-z0-9_-]+` and select `recipes/<name>.yaml`.
Without a name, the command reports selection instructions and available names
in sorted order. Recipe paths must stay inside the repository after resolving
symlinks. Loading errors go to stderr with exit code 2.

## Recipe editor support

The checked-in [V1 JSON Schema](schemas/recipe-v1.schema.json) provides field/value
completion and diagnostics for the current recipe format (link actions only).
With an editor using the YAML language server (`yaml-language-server`), add this
modeline at the top of `examples/recipes/links.yaml`:

```yaml
# yaml-language-server: $schema=../../schemas/recipe-v1.schema.json
version: 1
actions: []
```

The schema path is relative to the YAML file; adjust it for recipes elsewhere.
For a dotfiles repository, copy the schema locally and point to that copy.
Editor diagnostics supplement the authoritative Rust parser. JSON Schema cannot
detect duplicate YAML keys or enforce unique action IDs. YAML numeric syntax also
depends on the editor's YAML parser: JSON Schema treats `1.0` as an integer equal
to `1`, while the Rust parser rejects that YAML version value. Editor validation
does not check filesystem paths or execute actions.

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

`cargo test` also checks the editor schema and validates the existing YAML examples
against it. The JSON Schema validator is a development-only dependency with remote
and file fetching disabled.

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
