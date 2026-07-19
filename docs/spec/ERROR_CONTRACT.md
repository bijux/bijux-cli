---
title: Error Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Error Contract

Public DAG errors in `bijux-dag` must stay classifiable, scriptable, and
documented across human output, JSON output, and exit behavior.

## Scope

This contract governs:

- the public error code registry in `configs/dag/policy/error_codes.json`
- the operator reference page `docs/bijux-dag/interfaces/error-codes.md`
- the executable output and exit-code contract tests

## Registry rule

Public error codes are registry-governed identifiers. Each public code must
have:

- one stable identifier
- one named category
- one owning crate
- one durable description

## Governance rule

Public error code additions require docs plus test coverage.

The required documentation and test surfaces are:

- `docs/bijux-dag/interfaces/error-codes.md`
- `crates/bijux-dag-app/tests/error_output_contract.rs`
- `crates/bijux-dag-app/tests/error_exit_contract.rs`

## Compatibility rule

Reclassifying an existing public error, changing its meaning, or removing it
requires coordinated updates to the registry, this contract, the operator
reference page, and the linked contract tests in the same change.

## Related tests

- `crates/bijux-dag-app/tests/error_output_contract.rs`
- `crates/bijux-dag-app/tests/error_exit_contract.rs`
- `crates/bijux-dev/src/commands/contract_governance.rs`

## Versioning and change policy

Any incompatible change to public error code identity, category ownership, or
output/exit guarantees must update this contract and the linked docs and tests
in the same change.
