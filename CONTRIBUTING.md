# Contributing

Workstation is at the design and scaffold stage. Read the [design](docs/design.md)
and [plan](docs/plan.md) before implementing a feature. Discuss changes to recipe
semantics before introducing compatibility commitments.

Use stable Rust with rustfmt and Clippy, and keep compatibility with Rust 1.85.
Run the development checks in the README before submitting a change. Update the
design and examples whenever user-visible behavior changes.

Add focused behavior tests when implementing actions. Use temporary repositories
and target directories; tests must not provision the contributor's machine or
execute the personal dotfiles repository. Filesystem conflicts and interrupted
operations deserve explicit coverage.

Keep application-specific installers in example or user repositories. Add a
dependency only when it supports an implemented requirement. Commit Cargo.lock
changes with dependency updates.

Contributions are provided under the project's MIT license.
