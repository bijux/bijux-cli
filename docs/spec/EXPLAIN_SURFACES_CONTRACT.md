# Explain Surfaces Contract

## Scope
This contract defines stable explainability surfaces for operator diagnostics:
- `dag why-rerun`
- `dag why-cache-missed`
- `dag trace-artifact`
- run failure explain payloads

## Required behavior
- Explain output must be machine-readable and deterministic for identical inputs.
- Explain output must classify root causes into grouped dimensions when applicable.
- Explain output must gracefully handle partial/corrupt run directories.
- Unsupported backend or capability contexts must return explicit non-panicking failures.

## Drift explain requirements
- graph semantic drift must appear in explain cause groups
- environment drift must appear in explain cause groups
- artifact payload drift must appear in explain cause groups
- replay ancestry drift must appear in explain cause groups

## Output contract requirements
- JSON outputs must remain schema-lockstep with governed schemas and examples.
- Human-facing output must have concise and detailed governed examples.
- Wording drift between equivalent command families must be tracked.

## Performance requirement
Explain and diagnostics latency claims must be backed by generated benchmark reports.

## Authoritative test and governance surfaces
- `crates/bijux-dag-app/src/routes/diagnostics_routes.rs`
- `crates/bijux-dag-app/tests/diff_explain_contract.rs`
- `crates/bijux-dag-app/tests/replay_semantic_surface_contracts.rs`
- `crates/bijux-dag-app/tests/route_output_wording_snapshot_contracts.rs`
- `crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs`
