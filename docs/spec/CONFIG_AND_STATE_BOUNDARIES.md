# CONFIG AND STATE BOUNDARIES

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md
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

## SOURCE: docs/spec/CONFIG_CONTRACT.md
# Superseded by config cluster contract

- Superseded by: [CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md](./CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md)
- Appendix source: [appendices/config/CONFIG_CONTRACT.md](./appendices/config/CONFIG_CONTRACT.md)

## SOURCE: docs/spec/CONFIG_PRECEDENCE.md
# Superseded by config cluster contract

- Superseded by: [CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md](./CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md)
- Appendix source: [appendices/config/CONFIG_PRECEDENCE.md](./appendices/config/CONFIG_PRECEDENCE.md)

## SOURCE: docs/spec/CONFIG_PRECEDENCE_CONTRACT.md
# Superseded by config cluster contract

- Superseded by: [CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md](./CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md)
- Appendix source: [appendices/config/CONFIG_PRECEDENCE_CONTRACT.md](./appendices/config/CONFIG_PRECEDENCE_CONTRACT.md)

## SOURCE: docs/spec/CONFIG_STATE_BOUNDARIES.md
# Superseded by config cluster contract

- Superseded by: [CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md](./CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md)
- Appendix source: [appendices/config/CONFIG_STATE_BOUNDARIES.md](./appendices/config/CONFIG_STATE_BOUNDARIES.md)

## SOURCE: docs/spec/appendices/config/CONFIG_CONTRACT.md
# Config Contract

## Scope
Defines precedence and behavior for CLI args, config files, environment, and defaults.

## Precedence
`CLI args > explicit config file > environment > defaults`.

Default baseline source: `configs/dev/default_runtime_config.json`.

## Invariants
- Unknown config fields are rejected unless explicitly marked for compatibility handling.
- Semantically equivalent config values normalize to equivalent internal config.

## Related tests
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `evidence/battle/workflows/policy/*`

## Related schemas
- `configs/schema/runtime_config.schema.json`
- `configs/schema/policy_config.schema.json`

## Related docs
- `docs/spec/CONFIG_PRECEDENCE.md`
- `docs/spec/CONFIG_PRECEDENCE_CONTRACT.md`
- `docs/reference/CONFIG_INPUT_INVENTORY.md`
- `docs/spec/CONFIG_STATE_BOUNDARIES.md`
- `docs/spec/CONFIG_DEPRECATION.md`

## Versioning and change policy
Deprecated fields must include migration notes and validation behavior.

## SOURCE: docs/spec/appendices/config/CONFIG_PRECEDENCE.md
# Configuration Precedence

## Scope
Defines the single precedence table for effective configuration resolution.

## Precedence
`CLI > explicit config file > environment > defaults`

## Notes
- CLI values override all lower layers when provided.
- Explicit config file values override environment/default values.
- Environment values are only used for fields that are contractually env-addressable.
- Defaults are applied only when no higher layer provides a value.

## Related tests
- `crates/bijux-dag-app/tests/config_precedence_contract.rs`
- `crates/bijux-dag-app/tests/config_validation_contract.rs`

## Versioning and change policy
Any precedence change is breaking and requires docs + tests + drift-check updates in one change.

## SOURCE: docs/spec/appendices/config/CONFIG_PRECEDENCE_CONTRACT.md
# Config precedence contract

## Scope
Defines authoritative config sources, one precedence order, normalization behavior, and policy determinism requirements.

## Config source inventory

Runtime config sources used by product surfaces:

1. CLI flags and subcommand options
2. Explicit JSON config file provided by `--config`
3. Allowed environment inputs:
- `BIJUX_DAG_JOBS`
- `BIJUX_DAG_CACHE_MODE`
- `BIJUX_DAG_MATERIALIZE_INPUTS`
- `BIJUX_DAG_POLICY_JSON`
4. In-code defaults via `default_runtime_config`

## Precedence order

`CLI > explicit config file > environment > defaults`

This order is normative and must be consistent across run, replay, config show-effective, and policy show-effective surfaces.

## Effective resolution model

- Effective config must resolve into one normalized `RuntimeSurfaceConfig`.
- Unknown fields in explicit config must fail before execution.
- Malformed config files must fail before execution.
- Semantically equivalent configs must normalize identically.
- Effective config fingerprints must change for semantic config changes.

## Policy determinism

- `allow`/`deny` behavior must be deterministic under merged policy.
- `clean_env` and `allow_env` interactions must be explicit and deterministic.
- Policy evaluation trace must be available for operator/debug inspection.

## User-facing commands

- `dag config show-effective`
- `dag policy show-effective`

## Governance

- Config docs and precedence implementation must be updated in the same change.
- Ambient environment reads outside the allowed source inventory are forbidden.

## Versioning and change policy
Any precedence or source-set change is breaking and requires docs, tests, and drift guard updates in the same change.

## SOURCE: docs/spec/appendices/config/CONFIG_STATE_BOUNDARIES.md
# Config, Policy, Runtime State, and Artifacts

## Scope
Defines boundaries between configuration classes.

## Boundaries
- `config`: user-supplied static settings resolved before run start.
- `policy`: behavioral constraints enforced during planning/execution.
- `runtime state`: ephemeral in-memory execution status.
- `artifacts`: persisted run outputs, traces, and manifests.

## Invariants
- Config and policy are inputs to runtime behavior and must be representable in machine-readable forms.
- Runtime state is not a config source.
- Artifacts record outcomes, not unresolved policy/config intent.

## Related tests
- `crates/bijux-dag-app/tests/config_precedence_contract.rs`
- `crates/bijux-dag-app/tests/cache_invalidation_config_contract.rs`

## Versioning and change policy
Boundary changes require contract updates in CONFIG/POLICY/RUN_DIR docs and corresponding test updates.
