# Testing and test strategy

Audience: maintainers and contributors.
Owner: quality and release owners.
Status: stable.

## Test philosophy

Testing is organized by outward contract coverage rather than implementation detail.

## Crate and surface scopes

- `crates/bijux-dag-core/tests/`: parser compatibility, schema contracts, and core invariants.
- `crates/bijux-dag-runtime/tests/`: execution, replay, cache, and failure behavior.
- `crates/bijux-dag-app/tests/`: command contracts and output behavior.
- `crates/bijux-dag-cli/tests/`: parser/help surfaces, exit codes, and JSON envelope behavior.
- `crates/bijux-dev-dag/tests/`: repository architecture, policy, and release guardrails.

## Test classes

- `unit`: crate-local behavior checks.
- `contract`: schema/CLI/surface invariants and compatibility guarantees.
- `integration`: multi-module behavior with real artifacts.
- `conformance`: backend and protocol semantics across fixtures.
- `benchmark`: performance and equivalence evidence with retained baselines.

## Layout policy

- Root-level compatibility fixture docs should point to canonical crate layouts:
  - `crates/bijux-dag-core/tests/compat/v0.1`
  - `crates/bijux-dev-dag/tests/`
- Root-level architecture checks should remain in control-plane tooling crates, not duplicated at repo root.

## Governance rules

- Contracts should remain contract-oriented and not include implementation internals.
- Tests should map to explicit spec surfaces or maintainer references.
- Additional test directories require explicit owner approval in development governance documentation.

## Operational testing workflow

See `docs/operations/` for command-level operational workflows and `docs/dev/` for contributor workflow guidance.
