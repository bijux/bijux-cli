# Repo-wide Test Taxonomy

## Test Surface Areas

- `crates/bijux-dag-core/tests/`: structural invariants and parser compatibility.
- `crates/bijux-dag-runtime/tests/`: runtime execution, replay, and cache behavior contracts.
- `crates/bijux-dag-app/tests/`: CLI-facing application contracts and user-facing output envelopes.
- `crates/bijux-dag-cli/tests/`: public CLI parsing, help surfaces, and exit-code contracts.
- `crates/bijux-dev-dag/tests/`: repository architecture and policy checks.

## Contract Classes

- Syntax compatibility: fixed fixture contracts for spec parsing and fingerprinting.
- Execution contracts: deterministic node execution and manifest shape.
- Replay contracts: same input graph must produce same run graph fingerprint and node outputs.
- Cache contracts: repeatability and cache hit behavior under explicit cache modes.
- CLI contracts: help stability, JSON envelope shape, and invalid-argument behavior.
- Repository contracts: crate dependency layering and forbidden edge enforcement.

## Governance

- Runtime and app command tests should avoid implementation internals and focus on outward contracts.
- Additional test directories can only be added through explicit ownership decisions in `docs/DEVELOPMENT.md`.
