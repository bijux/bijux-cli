# System Health Diagnostics Documentation

## Health command surfaces

- `trust-health`: `crates/bijux-dev-dag/src/bin/trust_health.rs`
- storage diagnostics: `run_storage_health` in `crates/bijux-dev-dag/src/commands/mod.rs`
- drift dashboard: `run_drift_dashboard` in `crates/bijux-dev-dag/src/commands/mod.rs`

## Diagnostic domains

- runtime engine and scheduler diagnostics
- adapter and backend capability diagnostics
- artifact store, replay, bundle, and diff integrity diagnostics
- run history and provenance integrity diagnostics
- determinism and docs/config drift diagnostics

## Governance anchors

- contract: `docs/spec/SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md`
- suite: `configs/suites/system_health_verification.json`
- corpus: `evidence/cache/system_health/regression_corpus.json`
