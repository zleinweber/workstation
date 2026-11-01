# Workstation design

Status: initial proposal. This document describes intended behavior; the initial
repository contains only the Rust scaffold. Decisions can change before version 1.

## Purpose

Build a dotfiles manager with basic configuration management, distributed as a
standalone Rust executable. A user keeps configuration files, recipes, and custom
scripts in their own Git repository. The executable validates a selected recipe,
explains its actions, and applies it to the current machine.

The first user is the existing personal dotfiles repository. Success means that
its macOS and Codespaces setups can use the same generic executable without
embedding knowledge of Homebrew, Neovim, Atuin, or personal preferences in Rust.

The project is also an opportunity to learn Rust and develop a useful public tool.
It need not replace established tools for every user to be worth building.

## Scope

The first release supports Linux and macOS, local dotfiles repositories, YAML
recipes, package-based symlinks, direct file downloads, and executable scripts.
Its core promises are understandable configuration, predictable reruns, useful
errors, and a preview that does not execute recipe code.

Windows, remote machine provisioning, templates, secrets storage, package-manager
abstractions, archive extraction, a plugin ABI, parallel execution, automatic
upstream version discovery, and general rollback are deferred. Existing scripts
can perform application-specific work, including package-manager operations.

## Repository boundary

The framework repository owns the CLI, schema, generic actions, documentation,
tests, and release tooling. A user's repository owns recipes, configuration files,
installer scripts, package lists, and machine preferences.

Recommended user repository layout:

```text
~/.dotfiles/
  recipes/
    macos.yaml
    codespaces.yaml
  stow/
    zsh/.zshrc
    nvim/.config/nvim/init.lua
  scripts/
    install-tools.sh
  setup
```

`stow/` is a convention preserving compatibility with the existing layout. The
link action accepts a repository-relative source directory, so users can choose
another name. Git is a storage convention, not a runtime dependency; an ordinary
directory works too. Cloning and updating repositories remain the user's concern.

## CLI and configuration

The project and Cargo package are named Workstation (`workstation`). The
executable is `wstn`, with environment overrides prefixed `WSTN_`.

```sh
wstn setup macos --repo ~/.dotfiles
wstn describe codespaces --repo ~/src/dotfiles
wstn setup codespaces --dry-run
```

The recipe positional argument overrides `WSTN_RECIPE`. `--repo` overrides
`WSTN_REPO`, which defaults to `~/.dotfiles`. If no recipe is supplied, report how
to select one and list available recipes. Automatic platform selection belongs in
the user's wrapper initially; it should not silently select an unexpected recipe.

Recipe names match `[A-Za-z0-9_-]+` and resolve to `recipes/<name>.yaml`. Reject
path traversal and ambiguous extensions. Resolve a relative CLI repository path
against the invocation directory. Resolve action source paths and script working
directories against the repository, never the invocation directory. Expand only
leading `~/` in target paths; do not perform shell expansion or substitution.
Target paths must be absolute after expansion. Repository-owned source paths must
remain inside the repository after resolving symlinks.

`describe` parses and validates the selected recipe, then prints its description,
ordered actions, conditions, and resolved paths. It performs no execution,
downloads, credential prompts, or target writes. `setup --dry-run` additionally
inspects local filesystem state and reports planned operations and conflicts.
Scripts appear as opaque actions whose effects cannot be predicted. A preview
must distinguish unknown future state from an actual conflict.

Use exit status 0 for success, 1 for execution failure, and 2 for invalid CLI or
configuration. Write diagnostics to stderr and requested descriptions to stdout.
Do not print environment values that may contain credentials.

## Recipe schema

Use a top-level integer `version`, optional `description`, and ordered `actions`.
Require a unique `id` and explicit `type` for every action. Reject unknown keys,
duplicate YAML keys, unsupported versions, invalid types, and duplicate action
IDs before making changes. Publish a JSON Schema for editor assistance when the
parser lands; schema generation and runtime validation must agree.

Illustrative recipe, not yet executable:

```yaml
version: 1
description: Personal development environment
actions:
  - id: shell-config
    type: link
    source: stow/zsh
    target: "~/"
    conflict: error

  - id: editor-config
    type: link
    source: stow/nvim
    target: "~/"
    conflict: backup

  - id: install-tools
    type: script
    command: [bash, scripts/install-tools.sh, install]
    cwd: .
    when:
      os: [linux]
```

A download action has `url`, `target`, `sha256`, and optional `executable`
(default false). Require HTTPS and a full SHA-256 digest. Support optional
`when.os` values `linux` and `macos`, and `when.arch` values `x86_64` and `aarch64`
on all action types. Conditions are declarative, never shell expressions.
Separate actions can select platform-specific download URLs.

Keep recipes flat initially. Includes, inheritance, variables, and dependency
graphs need concrete use cases and explicit merge semantics before introduction.

## Execution model

Use a shared pipeline: resolve inputs, parse, validate, build ordered action
descriptions, inspect local state, execute, and summarize. `describe`, dry-run,
and setup must share resolution and validation rather than implement three
interpretations of recipes.

Validate all recipe structure and repository sources before mutation. Detect
overlapping declared destinations where possible. An arbitrary script can change
later state, so recheck each action immediately before applying it. Dry-run
reports this limitation; it is not a guarantee that later execution will match.

Run actions sequentially in recipe order. Stop at the first failure and summarize
completed, unchanged, skipped, failed, and unexecuted actions. Do not claim a
transaction across actions. Rerunning inspects current state rather than blindly
trusting a completion marker. Serialize setup runs for the same user using a lock
in the platform-appropriate user state directory.

## Link action

Mirror a source package's directory structure into the target using real
directories and symlinks for files. This is a documented subset of GNU Stow
behavior: no directory folding or claim of full Stow compatibility. Initially
reject source symlinks and special files with actionable diagnostics rather than
silently dereferencing or losing them.

An existing symlink to the intended source is unchanged. An existing real file,
broken symlink, or link to another source is a conflict. Do not follow target
ancestor symlinks outside the intended tree; fail and explain the conflicting
path. Also reject ancestor conflicts and two packages declaring the same target.

The default conflict policy is `error`. Opt-in `backup` moves the conflicting
entry to a unique backup location recorded in the run summary; never overwrite a
previous backup. If backup fails, leave the original in place and stop. Do not
silently delete conflicts. Use absolute source links initially and explain that
moving the repository requires rerunning setup.

Removal and automatic pruning of stale links are deferred. Before introducing
them, design an ownership manifest and ensure unrelated user files cannot be
removed. Switching recipes initially adds or updates requested configuration; it
does not uninstall everything omitted from the newly selected recipe.

## Download action

Skip when the destination is a regular file with the desired checksum and mode.
Otherwise fetch to a temporary file in the destination filesystem, verify the
checksum, set the requested executable permissions, and atomically rename into
place. Refuse symlink destinations and directory targets. A checksum mismatch or
interrupted download must leave the previous destination intact.

Use TLS verification, bounded redirects and timeouts, and preserve HTTPS on
redirects. Do not execute downloaded files as part of this action. Default to
error if an existing file has a different checksum; an explicit `replace: true`
authorizes replacing that exact destination after successful verification.
This permits deliberate upgrades without treating every existing binary as owned.

Archives and application-specific installation remain scripts for the first
release. Pinned downloads improve repeatability but do not make arbitrary scripts
or package-manager operations reproducible.

## Script action

Accept an argument array and invoke it directly, avoiding implicit shell parsing.
Users who need Bash explicitly select it, as in the example. Resolve `cwd`
relative to the repository, defaulting to its root. Support explicit environment
overrides while inheriting the process environment; values must not be logged.
Propagate failure and stream output. Child process cleanup on interruption is a
required part of the executor implementation.

Scripts run each time by default. An optional repository-independent absolute
`creates` path can skip execution when that path exists, with `~/` expansion.
This is a convenience guard, not proof of correctness or idempotence. Script
authors remain responsible for rerun safety. Do not run script-based checks in
describe or dry-run.

Recipes and scripts are trusted local code, not sandboxed content. Never request
sudo globally; scripts may explicitly request elevation when their work needs it.

## Rust architecture

Begin with one binary crate and small internal modules as features arrive:

- `cli`: arguments, help, environment precedence, and exit behavior.
- `config`: versioned recipe types, parsing, schema, and validation.
- `context`: repository, home, platform, and path resolution.
- `plan`: ordered actions and inspection results shared by all commands.
- `actions`: link, download, and script implementations.
- `runner`: sequential execution, locking, interruption, and summaries.

Use Rust 2024, commit Cargo.lock, forbid unsafe code in the application, and run
formatting, Clippy, and tests in CI. Add dependencies when needed: Clap for CLI
parsing, Serde plus a maintained YAML parser, an HTTP client with TLS support,
and focused filesystem/testing crates. Evaluate YAML parser maintenance and
duplicate-key handling before selecting it. Avoid an async runtime or dynamic
plugin system until requirements justify either.

Use temporary directories as fake repositories and homes in integration tests.
Never run a real user recipe as a routine test. Download tests use a controlled
fixture server; automated tests do not contact live installer endpoints.

## Bootstrap and Codespaces

Publish prebuilt binaries for Linux and macOS on x86_64 and aarch64. Users should
not need Cargo, Python, or GNU Stow to run the framework. User scripts can still
depend on those tools explicitly.

Retain a small executable `setup` in the personal dotfiles repository because
Codespaces discovers and invokes it. It determines the repository path from its
own location, selects the appropriate recipe, and invokes `wstn setup` with
quoted arguments. If the binary is absent, a documented bootstrap path downloads
a pinned release for the platform and verifies its checksum before execution.
Do not use a mutable `latest` URL as the reproducibility contract.

The bootstrap wrapper will be implemented after release artifacts exist. The
framework repository must not imply that this first scaffold is installable or
capable of configuring a machine.

## Migration and alternatives

First convert a small subset of the existing macOS or Codespaces recipe, keeping
application-specific installers as scripts. The Rust engine must not acquire
flags named after personal tools. Keep the old setup available until the new
recipe passes disposable-machine validation; then replace its orchestration
with the thin wrapper. Preserve the existing MIT licensing for reused code.

[Dotbot](https://github.com/anishathalye/dotbot) is the closest alternative for
YAML-directed linking and script execution. [Toml Bombadil](https://github.com/oknozor/toml-bombadil)
provides Rust-based dotfile profiles, templates, and hooks.
[chezmoi](https://www.chezmoi.io/) provides a broader multi-machine dotfiles system.
This project prioritizes an ordinary package layout, explicit setup actions, and
a standalone binary. Rust alone is not a unique feature or a guarantee of speed.

## Open decisions

- Public repository location.
- Whether recipe auto-selection should eventually be configurable in the engine.
- Whether the first public release needs unlink and ownership tracking.
- Whether archive downloads are common enough to become a built-in action.
- Exact user state and backup directory conventions across Linux and macOS.
- Release signing and installer delivery mechanism after the first working slice.

Resolve these against actual migration experience; do not block the parser and
link prototype on features deliberately deferred above.
