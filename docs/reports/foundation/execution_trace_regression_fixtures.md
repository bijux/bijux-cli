# Execution Trace Regression Fixtures

## Fixture anchors

- cancellation trace fixture: `crates/bijux-dag-runtime/tests/fixtures/state_machine/cancellation_trace.json`
- evolution trace fixture: `crates/bijux-dag-runtime/tests/fixtures/state_machine/evolution_trace.json`
- run-directory trace files: `nodes/<node_id>/trace.json` under run fixtures

## Regression classes

- ordering determinism and timestamp coherence
- successful/failed/cancelled trace completeness
- trace schema stability and corruption detection
- replay inspection compatibility
