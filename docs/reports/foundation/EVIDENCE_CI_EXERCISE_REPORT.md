# Evidence Commands CI Exercise Report

Execution sources checked:
- `.github/workflows/rust-test.yml`
- `make/root.mk`
- `make/evidence.mk`
- `configs/policy/evidence_command_classification.json`

## Exercised in `make test-all`
- `verify evidence-battle`
- `verify evidence-cache`
- `verify evidence-replay`
- `verify evidence-compat`
- `verify evidence-fault`
- `verify evidence-perf`
- `verify evidence-consumers`
- `verify evidence-release-set`

## Advisory-only (not blocking in `make test-all`)
- `verify evidence-compare`

## Currently not exercised in `make test-all`
- `verify evidence-schema`
- `verify evidence-registry`
- `verify evidence-authoring`
- `verify evidence-drift`
- `verify evidence-foundation`

These remain available through dedicated wrappers (`make evidence-*`) and release/governance workflows.
