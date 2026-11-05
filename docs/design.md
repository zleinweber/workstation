# Workstation design

Status: proposal. Only the Rust scaffold exists; the behavior below is planned.

## Purpose and scope

Workstation is a standalone Rust executable for repeatable Linux and macOS
workstation setup. It applies YAML recipes to a local dotfiles directory, keeping
personal configuration and installers separate from the reusable engine.

Version 1 provides file symlinks, verified downloads, and script execution, with
predictable reruns, useful errors, and previews that never execute recipe code.
Application-specific work, including package management, belongs in user scripts.

Deferred: Windows, remote provisioning, templates, secrets storage, package-manager
adapters, archives, plugins, parallel execution, upstream version discovery,
general rollback, and recipe composition.

## Terminology and repository layout

- **Workstation** (`workstation`): the project and Cargo package.
- **`wstn`**: the command-line executable, pronounced “workstation.”
- **Dotfiles directory**: user-owned input, defaulting to `~/.dotfiles`. Git is
  optional; cloning and updates are the user's responsibility.
- **Recipe**: a named YAML file containing ordered setup actions.
- **Bundle**: configuration files whose directory structure is mirrored into a
  target directory using symlinks.
- **Script**: a user-owned program invoked by a recipe.

The framework owns the CLI, schema, generic actions, tests, and release tooling.
The dotfiles directory owns recipes, bundles, scripts, and machine preferences:

```text
~/.dotfiles/
  recipes/
    macos.yaml
    codespaces.yaml
  bundles/
    zsh/.zshrc
    nvim/.config/nvim/init.lua
  scripts/
    install-tools.sh
  setup
  wstn.toml                  # Optional framework configuration
```

`bundles/` is a convention. A link action names its repository-relative source,
so existing `stow/` layouts still work. Configurable top-level directory names
are intended; their configuration keys remain an open decision.

## CLI and configuration

```sh
wstn setup macos --repo ~/.dotfiles
wstn describe codespaces --repo ~/src/dotfiles
wstn setup codespaces --dry-run
```

Configuration precedence is CLI arguments, then `WSTN_` environment variables,
then the dotfiles directory's optional `wstn.toml`, then built-in defaults.
Resolve the repository first using `--repo`, `WSTN_REPO`, or `~/.dotfiles`, then
read its configuration file. The file cannot select its own repository location.
The recipe argument overrides `WSTN_RECIPE`; the TOML recipe key is still to be
specified. If no recipe is selected, explain how to select one and list available
recipes. Platform auto-selection belongs in the user's wrapper initially.

| Command | Behavior |
| --- | --- |
| `describe` | Validate the recipe; print its description, ordered actions, conditions, and resolved paths. |
| `setup --dry-run` | Also inspect local state and report planned operations and conflicts. |
| `setup` | Apply actions sequentially and summarize the result. |

Both previews avoid execution, downloads, credential prompts, and target writes.
Scripts have unknown effects; previews must distinguish unknown future state from
known conflicts.

Exit codes: 0 for success, 1 for execution failure, 2 for invalid CLI or
configuration. Send diagnostics to stderr and requested descriptions to stdout.
Never log environment values that may contain credentials.

## Recipes

### Resolution and validation

Recipe names match `[A-Za-z0-9_-]+` and resolve to `recipes/<name>.yaml` by default.
Reject path traversal and ambiguous extensions. Resolve relative repository paths
against the invocation directory; resolve action sources and script working
directories against the repository. Repository-owned paths must remain inside
that repository after symlink resolution.

Target paths must be absolute after expanding a leading `~/`. Perform no other
shell expansion or substitution.

Recipes require an integer `version`, optional `description`, and ordered
`actions`. Each action requires a unique `id` and explicit `type`. Reject unknown
keys, duplicate YAML keys or action IDs, unsupported versions, and invalid types
before mutation. Publish an editor JSON Schema alongside the parser and keep it
consistent with runtime validation.

### Example

Illustrative only; recipes are not implemented yet:

```yaml
version: 1
description: Personal development environment
actions:
  - id: shell-config
    type: link
    source: bundles/zsh
    target: "~/"
    conflict: error

  - id: editor-config
    type: link
    source: bundles/nvim
    target: "~/"
    conflict: backup

  - id: install-tools
    type: script
    command: [bash, scripts/install-tools.sh, install]
    cwd: .
    when:
      os: [linux]
```

All actions support declarative `when.os` (`linux`, `macos`) and `when.arch`
(`x86_64`, `aarch64`) conditions. Separate actions can select platform-specific
URLs. Recipes remain flat until composition has concrete use cases and defined
merge semantics.

## Execution

All commands share input resolution, parsing, validation, and ordered action
planning. Dry-run and setup also inspect local state; setup then executes and
summarizes.

Validate recipe structure and repository sources before mutation, detecting
overlapping destinations where possible. Recheck each action before applying it:
scripts can change later state, so dry-run cannot guarantee an identical run.

Execute in recipe order and stop at the first failure. Summarize completed,
unchanged, skipped, failed, and unexecuted actions. Earlier changes remain after
a failure. Reruns inspect current state rather than trust completion markers.
Serialize setup runs per user with a lock in the platform's user state directory.

## Actions

### Link

Mirror a bundle into its target with real directories and absolute file symlinks.
This is a subset of GNU Stow behavior, without directory folding. Reject source
symlinks and special files with actionable errors.

A symlink to the intended source is unchanged. Real files, broken symlinks, and
links elsewhere are conflicts. Reject conflicting ancestors, target ancestor
symlinks that escape the intended tree, and bundles declaring the same destination
file. Bundles may share parent directories, as both examples above share `~/`.

The default `conflict: error` stops on a conflict. Opt-in `conflict: backup` moves
the conflicting entry to a unique location recorded in the summary, without
overwriting earlier backups. If backup fails, preserve the original and stop.
Moving the repository requires rerunning setup because links are absolute.

Switching recipes does not remove omitted configuration. Unlink and stale-link
pruning require a future ownership manifest that protects unrelated files.

### Download

Require `url` (HTTPS), `target`, and a full `sha256` digest. Optional `executable`
and `replace` both default to false.

Skip a regular file with the desired checksum and mode. A different checksum is
a conflict unless `replace: true` authorizes replacing that destination. Refuse
symlink and directory destinations.

Fetch into a temporary file on the destination filesystem, verify its checksum,
set executable permissions, and atomically rename it into place. Failed or
interrupted downloads must preserve the previous destination. Use TLS
verification, bounded timeouts and redirects, and HTTPS throughout redirects.
Never execute the downloaded file.

Archives and application-specific installation remain scripts. Pinned downloads
do not make scripts or package-manager operations reproducible.

### Script

Invoke the `command` argument array directly; shell users must explicitly select
a shell. Resolve `cwd` against the repository, defaulting to its root. Inherit the
process environment and support explicit overrides without logging their values.
Stream output, propagate failures, and clean up child processes on interruption.

Scripts run on every setup unless an optional absolute `creates` path exists
(with leading `~/` expansion). This guard does not prove correctness; authors
remain responsible for rerun safety. Previews never run script-based checks.

Recipes and scripts are trusted local code. Scripts may request elevation;
the engine never requests sudo globally.

## Implementation and validation

Start with one Rust 2024 binary crate and add internal modules as needed:

| Module | Responsibility |
| --- | --- |
| `cli` | Arguments, help, configuration precedence, exit behavior |
| `config` | Versioned types, parsing, schema, validation |
| `context` | Repository, home, platform, path resolution |
| `plan` | Shared ordered actions and inspection results |
| `actions` | Link, download, and script implementations |
| `runner` | Execution, locking, interruption, summaries |

Commit Cargo.lock, forbid unsafe application code, and run formatting, Clippy,
and tests in CI. Add dependencies as features need them: Clap, Serde with a
maintained YAML parser, a TLS-capable HTTP client, and focused filesystem/test
crates. Check YAML maintenance and duplicate-key handling before choosing a
parser. Defer an async runtime until requirements justify it.

Tests use temporary repositories and homes, never real user recipes. Download
tests use a controlled fixture server rather than live installer endpoints.
Milestones and acceptance checks live in [the implementation plan](plan.md).

## Release and migration

Publish Linux and macOS binaries for x86_64 and aarch64. Running the framework
must not require Cargo, Python, or GNU Stow; scripts declare their own dependencies.

Keep a small executable `setup` in the dotfiles directory for Codespaces. It
finds the repository from its own location, selects a recipe, and invokes
`wstn setup` with quoted arguments. Once release artifacts exist, a bootstrap
path can download a pinned binary and verify its checksum before execution.
Mutable `latest` URLs are not a reproducibility contract.

Migrate a small macOS or Codespaces slice first, leaving application installers
in scripts. Keep the old setup until disposable-machine validation passes, then
replace its orchestration with the wrapper. Preserve MIT licensing for reused
code. Until release artifacts exist, documentation must identify the scaffold
as incomplete.

## Alternatives

[Dotbot](https://github.com/anishathalye/dotbot) provides YAML-directed linking and
scripts. [Toml Bombadil](https://github.com/oknozor/toml-bombadil) offers Rust-based
profiles, templates, and hooks. [chezmoi](https://www.chezmoi.io/) provides broader
multi-machine dotfiles management. Workstation prioritizes an ordinary bundle
layout, explicit setup actions, and a standalone executable.

## Open decisions

- `wstn.toml` schema, including recipe defaults and directory configuration.
- Future engine support for recipe auto-selection.
- Whether the first public release needs ownership tracking and unlink.
- Whether common archive workflows justify a built-in action.
- Platform-specific state and backup locations.
- Release signing and installer delivery.

Resolve these through migration experience while proceeding with the parser and
link prototype.
