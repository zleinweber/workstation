# Workstation design

Status: initial implementation. The internal parser supports the V1 recipe
envelope and link action fields. CLI recipe loading and execution remain planned.

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

Recipes contain ordered actions. The initial V1 definition is:

| Field | Type | Required | Validation |
| --- | --- | --- | --- |
| `version` | integer | yes | Must equal `1`; quoted strings and floats are invalid. |
| `description` | string | no | If present, must contain non-whitespace text; null is invalid. |
| `actions` | list of actions | yes | Execution order; an empty list is valid. |
| action `id` | string | yes | Must contain non-whitespace text and be unique within the recipe. |
| action `type` | string | yes | Selects a supported action type and its fields. |

IDs are compared exactly and case-sensitively. Valid strings are preserved
without trimming. Unknown fields, duplicate YAML keys, unsupported versions,
unsupported action types, and incorrect field types are errors. Input must be a
single YAML document. Duplicate action ID errors identify both action positions.

The first parser increment supports `type: link`, with required non-whitespace
strings `source` and `target`, and optional `conflict` (`error` by default, or
`backup`). This validates content only: path resolution, containment, existence,
and filesystem conflict checks remain separate work. Script, download, and
`when` fields described below are proposals and are not accepted by this parser
yet. The internal entry point is `config::parse_recipe`; CLI loading and editor
JSON Schema are deferred.

V1 can grow through new action types and optional fields whose defaults preserve
existing recipes' meaning. Breaking changes, such as removing fields, requiring
new fields on existing actions, or changing existing defaults or semantics,
normally require a new schema version. Older binaries may reject newer V1
features; schema version and binary version are distinct. While experimental,
breaking changes may still be documented within V1 without a stability promise.

### Example

Illustrative full recipe; script and condition support are not implemented yet.
Valid parser inputs live in `examples/recipes/`:

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

The first parser uses `serde_yaml_ng` 0.10, a maintained fork of `serde_yaml`
with a declared Rust 1.64 minimum (below this project's 1.85 minimum). Parse
through its YAML value representation before typed deserialization to reject
duplicate keys throughout the document and avoid coercing numbers into strings.
Focused tests lock down duplicate-key rejection and strict field types,
including rejecting null for the required action list.

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
