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

- `docs/reports/foundation/archive/REPLAY_HARDENING_REPORT.md`
- `docs/reports/foundation/archive/CACHE_HARDENING_REPORT.md`
- `docs/reports/foundation/archive/RUN_DIR_IMPORT_EXPORT_HARDENING_REPORT.md`
- `docs/reports/foundation/archive/CONFIG_POLICY_DETERMINISM_REPORT.md`
- `configs/policy/battle_trust_properties.json`
