# bijux-dev-cli

`bijux-dev-cli` is the maintainer control-plane crate for `bijux dev cli ...` command workflows.

## Scope

- Owns maintainer-facing automation orchestration.
- Owns maintainer-facing report assembly.
- Keeps runtime command law and runtime state mutation rules in runtime crates.

## Non-Goals

- Defining runtime command law.
- Becoming a second executable.
- Replacing the canonical `bijux` binary entrypoint.
