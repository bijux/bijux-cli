# Configuration, state, and boundaries contract

**What this spec is not**: operator onboarding, performance tuning guidance, or historical design debate.

## Scope

This contract defines:

- config source precedence and normalization
- runtime/policy/artifact boundaries
- config deprecation controls
- stable contract changes and change impact

## Canonical rule set

- Effective precedence is `CLI > explicit config file > environment > defaults`.
- Runtime state is ephemeral and may not be treated as configuration source.
- Policy controls behavior, while artifacts record outcomes.
- Deprecated configuration fields remain parseable only within declared compatibility windows.

## Determinism and compatibility requirements

- Precedence and normalization behavior is deterministic for equivalent inputs.
- Changes to precedence or state boundaries are breaking and require test and fixture updates.
- Boundaries are enforced in execution planning, policy inspection, and verification surfaces.

## Implementation and evidence links

- Source configuration schemas: `configs/schema/runtime_config.schema.json`, `configs/schema/policy_config.schema.json`.
- Tests: `crates/bijux-dag-app/tests/config_precedence_contract.rs`, `crates/bijux-dag-app/tests/config_validation_contract.rs`.
- Evidence and governance artifacts: config and policy sections in `docs/reference`.
