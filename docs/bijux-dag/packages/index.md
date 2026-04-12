# DAG Packages

This section is the package ownership map for DAG behavior. Use it to locate the right package before making changes to graph modeling, execution policy, artifact contracts, or DAG command orchestration.

## Package Map

| Package | Owns | Enter Here When |
| --- | --- | --- |
| [`bijux-dag-core`](bijux-dag-core.md) | Graph truth, planner lowering, deterministic core semantics | the issue is graph model rules, planning invariants, or semantic correctness |
| [`bijux-dag-runtime`](bijux-dag-runtime.md) | Runtime policy, execution flow, replay behavior, diagnostics boundaries | the issue is run behavior, replay parity, lifecycle orchestration, or runtime guarantees |
| [`bijux-dag-app`](bijux-dag-app.md) | Command orchestration, user-facing shaping, app-layer wiring | the issue is orchestration flow, command composition, or top-level request handling |
| [`bijux-dag-cli`](bijux-dag-cli.md) | Thin CLI entrypoint wrapper for DAG command surfaces | the issue is DAG CLI entrypoint wiring or executable boundary behavior |
| [`bijux-dag-artifacts`](bijux-dag-artifacts.md) | Artifact identity, integrity semantics, and artifact lifecycle helpers | the issue is artifact schema, identity, storage contract, or integrity checks |
| [`bijux-dag-testkit`](bijux-dag-testkit.md) | Shared deterministic fixtures and test support surfaces | the issue is shared fixtures, deterministic test inputs, or common test helpers |

## Navigation Rule

Choose the package page based on ownership first. If a change touches two rows in the table, treat it as an explicit cross-package boundary change and validate both contracts.
