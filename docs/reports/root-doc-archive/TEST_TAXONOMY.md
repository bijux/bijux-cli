# Repo-wide Test Taxonomy

## Test Surface Areas

- `crates/bijux-dag-core/tests/`: structural invariants and parser compatibility.
- `crates/bijux-dag-runtime/tests/`: runtime execution, replay, and cache behavior contracts.
- `crates/bijux-dag-app/tests/`: CLI-facing application contracts and user-facing output envelopes.
- `crates/bijux-dag-cli/tests/`: public CLI parsing, help surfaces, and exit-code contracts.
- `crates/bijux-dev-dag/tests/`: repository architecture and policy checks.

## Contract Classes

- unit: crate-local behavior checks.
- contract: schema/CLI/surface invariants and compatibility guarantees.
- integration: multi-module behavior with real artifacts.
- conformance: backend and protocol semantics across fixtures.
- benchmark: performance/equivalence quality measurements with evidence artifacts.

## Governance

- Runtime and app command tests should avoid implementation internals and focus on outward contracts.
- Additional test directories can only be added through explicit ownership decisions in `docs/DEVELOPMENT.md`.
