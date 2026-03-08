# Release evidence report

## Scope

Aggregates high-trust release evidence and rejects readiness based on raw test counts alone.
raw test totals are insufficient

## Required evidence surfaces

- battle scenario enforcement: `battle-suite-mandatory`
- replay hardening: `replay-contract`
- cache hardening: `cache-evolution`
- run directory verification and import/export hardening: `artifact-hardening`
- config and policy determinism: `config-policy-determinism`

## Readiness rule

Release readiness depends on the required evidence surfaces above. Raw test totals are insufficient without trust-property coverage on these surfaces.

## Evidence links

- `docs/reports/foundation/archive/replay_hardening_report.md`
- `docs/reports/foundation/archive/cache_hardening_report.md`
- `docs/reports/foundation/archive/run_dir_import_export_hardening_report.md`
- `docs/reports/foundation/archive/config_policy_determinism_report.md`
- `configs/policy/battle_trust_properties.json`
