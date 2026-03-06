# Architecture

## Ownership boundaries

- `crates/bijux-dag-core`: parsing, model definitions, validation, and graph algorithms.
- `crates/bijux-dag-runtime`: scheduler, cache model, execution engine, and trace/manifest output.
- `crates/bijux-dag-app`: user-facing command behavior and orchestration for runtime operations.
- `crates/bijux-dag-cli`: umbrella command wiring and top-level UX.
- `crates/bijux-dev-dag`: repository control-plane for checks, tests, release workflows.
- `dag-api` (planned boundary): service control-plane for typed registry/schedule/run/artifact APIs.

## Execution backend baseline

- Local backend is the correctness baseline for deterministic behavior.
- Other backends must satisfy the same execution contract and acceptance gates before production use.

## Module boundaries

- `core` contains no I/O dependencies on other runtime crates.
- `runtime` must not depend on app/cli types.
- `app` may depend on runtime and artifacts for command implementation.
- `cli` depends on app for command registration.
- `dev` depends on no policy-sensitive runtime internals.
- future `dag-api` must depend on typed runtime/control-plane contracts, not CLI internals.

## Layered command flow

1. Operator invokes `bijux` (`bijux-dag-cli`).
2. `bijux-dag-app` resolves the selected DAG command and executes.
3. `bijux-dag-runtime` performs node execution and creates run artifacts.
4. `bijux-dev-dag` coordinates cross-repo checks and release gates via suites.
