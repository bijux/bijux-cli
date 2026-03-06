# Architecture

## Ownership boundaries

- `crates/bijux-dag-core`: parsing, model definitions, validation, and graph algorithms.
- `crates/bijux-dag-runtime`: scheduler, cache model, execution engine, and trace/manifest output.
- `crates/bijux-dag-app`: user-facing command behavior and orchestration for runtime operations.
- `crates/bijux-dag-cli`: umbrella command wiring and top-level UX.
- `crates/bijux-dev-dag`: repository control-plane for checks, tests, release workflows.

## Module boundaries

- `core` contains no I/O dependencies on other runtime crates.
- `runtime` must not depend on app/cli types.
- `app` may depend on runtime and artifacts for command implementation.
- `cli` depends on app for command registration.
- `dev` depends on no policy-sensitive runtime internals.

## Layered command flow

1. Operator invokes `bijux` (`bijux-dag-cli`).
2. `bijux-dag-app` resolves the selected DAG command and executes.
3. `bijux-dag-runtime` performs node execution and creates run artifacts.
4. `bijux-dev-dag` coordinates cross-repo checks and release gates via suites.
