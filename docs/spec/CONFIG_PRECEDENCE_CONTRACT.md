---
title: Config Precedence Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Config Precedence Contract

`bijux-dag` configuration must resolve deterministically so operators can
explain why one runtime setting won over another before execution starts.

## Scope

This contract governs effective configuration resolution for:

- `dag config show-effective`
- `dag policy show-effective`

It also governs the merge order between defaults, environment-derived config,
explicit config files, and CLI overrides.

## Canonical precedence

CLI > explicit config file > environment > defaults

This ordering applies to runtime config fields such as `jobs`, `cache_mode`,
`materialize_inputs`, and policy fields when the CLI surfaces expose them.

## Validation rules

- Unknown fields in explicit config must fail before execution.
- Malformed config files must fail before execution.
- Effective config must be inspectable without starting a workflow.
- Policy evaluation trace must be available for operator/debug inspection.

## Operator surfaces

- `dag config show-effective` must show the merged runtime config
- `dag policy show-effective` must show the effective policy plus evaluation
  trace

## Related tests

- `crates/bijux-dag-app/tests/config_precedence_contract.rs`
- `crates/bijux-dag-app/tests/config_validation_contract.rs`
- `crates/bijux-dag-app/tests/config_effective_command_contract.rs`

## Versioning and change policy

Any incompatible change to configuration precedence, config-file validation, or
effective-config command behavior must update this contract and the linked tests
in the same change.
