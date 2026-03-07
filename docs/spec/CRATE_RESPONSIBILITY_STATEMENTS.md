# Crate Responsibility Statements

## `bijux-dag-core`
- DAG schema, parsing, canonicalization, validation, and semantic graph rules.
- No CLI or runtime execution orchestration.

## `bijux-dag-artifacts`
- Run directory schemas, artifact models, path normalization, integrity metadata, retention helpers.
- No scheduler or node execution policy logic.

## `bijux-dag-runtime`
- Execution engine, scheduler semantics, state transitions, policy enforcement during execution.
- Consumes core + artifacts contracts, but does not define CLI UX.

## `bijux-dag-app`
- Command orchestration, user-facing command output shaping, inspect/verify command behavior.
- No scheduler internals or adapter execution implementation.

## `bijux-dag-cli`
- Thin binary wrapper over app command tree.
- No runtime semantics.

## `bijux-dag-testkit`
- Shared workspace test helpers and fixture utilities.

## `bijux-dev-dag`
- Repository control-plane checks, governance suites, drift and boundary verification.
