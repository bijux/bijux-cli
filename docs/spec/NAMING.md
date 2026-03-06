# Naming Conventions

## Goals
- Make the layer and intent obvious from the name.
- Prefer domain-specific names over generic ones.
- Keep names stable to avoid unnecessary churn.

## Crates
- `bijux-dag-core`: spec, validation, canonicalization, fingerprints, resolver (pure).
- `bijux-dag-artifacts`: run directory layout, schemas, read/write helpers.
- `bijux-dag-runtime`: planner + engine, adapters, scheduling, cache.
- `bijux-dag-app`: CLI wiring only (no business logic).
- `bijux-dag-cli`: umbrella CLI (dispatches sub-apps).

## Modules
- Avoid catch-all names like `utils`, `common`, `helpers`, `ops`.
- Modules should map to a domain (e.g., `planner`, `cache`, `adapter`, `artifacts`).
- Keep modules small and focused; split by domain when they grow.

## Banned Names
- `utils`, `common`, `helpers`, `ops`
- Generic `Context` without a prefix (use `RunContext`, `ParseContext`, etc.)
- Generic `Config` without a prefix (use `RuntimeConfig`, `PolicyConfig`, etc.)

## Types
- Prefer explicit names: `RuntimeConfig`, `PolicyConfig`, `RunContext`.
- Avoid ambiguous `Config`, `Context`, `Result` in public APIs.
- Names should encode the layer (e.g., `NodeTrace` in artifacts, `ExecutionPlan` in runtime).

## Fields
- Use snake_case for JSON fields and Rust struct fields.
- Prefer semantic names over generic: `node_fingerprint`, `cache_mode`, `graph_snapshot`.

## Commands
- CLI verbs should be short and consistent: `validate`, `run`, `replay`, `diff`, `explain`.
- Prefer `node` over `inspect` for per-node details.
- Prefer `verify` over `verify-run`.

## Files
- Specs live under `docs/spec/` with `v0.1` in the filename when versioned.
- Architecture and ADRs live under `docs/architecture/` and `docs/ADRs/`.
