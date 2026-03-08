# Config policy determinism report

## Scope

Captures deterministic config precedence, policy evaluation behavior, and governance evidence.

## Canonical precedence

The repository enforces one precedence order:

- CLI overrides
- explicit config file
- allowed environment inputs
- default runtime config

Contract authority:

- `docs/spec/CONFIG_PRECEDENCE_CONTRACT.md`

## Determinism guarantees

Required guarantees:

- semantically equivalent config inputs normalize to the same effective runtime config
- unknown config fields fail before execution
- malformed config payloads fail before execution
- `clean_env` and allowlist behavior stay deterministic
- ambient environment reads outside approved keys are rejected by governance checks

## Operator/debug surfaces

Required command surfaces:

- `dag config show-effective`
- `dag policy show-effective`

These surfaces must reflect the same precedence and policy merge semantics as execution paths.

## Trust-property linkage

Config and policy determinism protects battle trust property `tp_config_policy_determinism`.
This trust property remains mandatory release evidence in foundation verification.
