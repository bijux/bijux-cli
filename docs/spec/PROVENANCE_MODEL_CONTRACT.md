# Provenance Model Contract

## Scope
This contract defines provenance semantics for run, node, and artifact surfaces.
It covers lineage traversal, replay/import continuity, and operator explain outputs.

Authoritative implementation surfaces:
- `crates/bijux-dag-artifacts/src/lifecycle/lineage.rs`
- `crates/bijux-dag-artifacts/src/layout/platform.rs`
- `crates/bijux-dag-app/src/lib.rs` (`inspect_artifact`)
- `crates/bijux-dag-runtime/src/artifacts/storage/semantic_lineage.rs`

## Provenance graph model
- run -> node -> artifact relationships must be representable and queryable
- artifact provenance includes producer run identity and node identity
- lineage edges define upstream and downstream artifact dependencies
- replayed and imported runs retain source-run lineage continuity

## Completeness guarantees
- completed runs: provenance and lineage fields are required for inspect surfaces
- failed and cancelled runs: provenance continuity must remain queryable
- partial reruns: lineage relations for produced outputs must remain explicit

## Determinism guarantees
- repeated provenance traversal over identical data is deterministic
- provenance serialization outputs are stable for unchanged lineage snapshots
- query results for upstream/downstream traversal are ordering-stable

## Explain and schema requirements
- machine output schema: `configs/schema/operator/artifact_trace.schema.json`
- inspect output schema: `configs/schema/operator/artifact_inspect.schema.json`
- human operator examples must include provenance and lineage sections

## Stress and performance requirements
- provenance queries must remain bounded on large lineage snapshots
- latency evidence must be recorded in generated provenance reports

## Stability level
Stable governance contract for `v0.1` release surfaces.
