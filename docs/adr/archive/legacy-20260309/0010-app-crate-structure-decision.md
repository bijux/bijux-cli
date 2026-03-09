# ADR 0010: Keep `bijux-dag-app` as a single orchestration crate with internal command modules

## Status
Accepted

## Date
2026-03-07

## Context
The current workspace has a dedicated binary crate (`bijux-dag-cli`) and an application crate (`bijux-dag-app`) that contains command parsing, dispatch, and orchestration logic. The proposed alternative is a split into an additional crate such as `bijux-dag-runbook` or `bijux-dag-commands`.

We need to reduce boundary churn while repairing module ownership and testability.

## Decision
Keep `bijux-dag-app` as a single crate and split it into explicit internal modules:
- `commands`
- `format`
- `read`
- `write`
- `explain`
- `graph`
- `cache`
- `replay`
- `migrate`

Do not introduce a second app-level crate now.

## Rationale
- Keeps dependency graph stable while boundary contracts are being enforced.
- Avoids duplicating command model types across crates.
- Preserves a thin `bijux-dag-cli` binary crate with wiring only.
- Allows stronger internal visibility control using `pub(crate)` defaults.

## Consequences
- `bijux-dag-app` must stay orchestration-only and avoid low-level runtime internals.
- Clap command structures must live under `src/commands/`.
- Any future split into multiple app crates requires a new ADR with measured API and maintenance impact.
