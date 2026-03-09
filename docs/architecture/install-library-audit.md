# Install Library Audit

Date: 2026-03-09

## Scope

Audited crate: `crates/bijux-cli-install/src/lib.rs` and module boundaries.

## Module ownership

- `metadata.rs`: ecosystem/channel/package strategy contracts.
- `paths.rs`: path discovery, path-shadowing primitives, first-run marker behavior.
- `diagnostics.rs`: install health report assembly.
- `completion.rs`: completion script generation, completion target paths, shell detection, compatibility notes.
- `compatibility.rs`: config/history/plugin path precedence and dotenv parsing/writing.
- `state.rs`: lock management, state initialization helpers, migration hook.

## Boundary decisions

1. `bijux-cli-install` owns compatibility path and install diagnostics primitives.
2. Higher-level command behavior stays in `bijux-cli-core`.
3. Python bridge reuses install primitives rather than duplicating filesystem logic.
