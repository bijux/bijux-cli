# Workspace crate dependency graph

```text
bijux-dag-cli
  -> bijux-dag-app
    -> bijux-dag-core
    -> bijux-dag-runtime
    -> bijux-dag-artifacts

bijux-dag-runtime
  -> bijux-dag-core
  -> bijux-dag-artifacts

bijux-dev-dag
  -> bijux-dag-core

bijux-dag-core
  -> (workspace shared libs only)

bijux-dag-artifacts
  -> (workspace shared libs only)
```

## Boundary intent
- `bijux-dag-cli` is binary wiring only.
- `bijux-dag-app` owns user command orchestration and output shaping.
- `bijux-dag-runtime` owns execution and adapter runtime behavior.
- `bijux-dag-core` owns DAG model/parse/validate/resolve semantics.
- `bijux-dag-artifacts` owns artifact schema models and artifact persistence APIs.
- `bijux-dev-dag` owns repository governance and validation tooling.

## Forbidden direct edges
- `bijux-dag-runtime` -> `bijux-dag-app`
- `bijux-dag-runtime` -> `bijux-dag-cli`
- `bijux-dag-core` -> `bijux-dag-runtime`
- `bijux-dev-dag` -> runtime internal crates
