# ADR: app router target architecture

## Status
Accepted

## Context
`crates/bijux-dag-app/src/lib.rs` became a routing hot spot. Command family logic, path handling, and response shaping in one file slowed review and coverage growth.

## Decision
The app crate routing model is split by command family under `src/routes`:
- `validate_routes`, `plan_routes`, `run_routes`, `inspect_routes`, `runs_routes`, `replay_routes`, `diff_routes`, `prove_verify_routes`, `export_import_routes`, and `surface_routes`.
- Shared routing concerns live in dedicated modules: output selection, preconditions, path resolution, run lookup, and response shaping.
- `lib.rs` remains the command dispatch entrypoint only.

## Consequences
- Routing ownership is explicit per command family.
- Helper modules can be directly unit-tested without full CLI integration.
- File-size and routing coverage targets are enforceable through policy and contract tests.
