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
