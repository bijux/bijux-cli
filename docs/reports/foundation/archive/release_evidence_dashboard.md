# Release Evidence Dashboard

Source: `docs/reports/foundation/release_critical_evidence_matrix.md`

## Release-Critical Families
- battle
- cache
- replay (under cache)
- compat
- fault
- operator
- perf
- consumers governance
- release-set governance

## Blocking Lane Contract
- full lane (`make test-all`) blocks on release-critical verify commands
- `evidence-all` runs full evidence verification entrypoint
