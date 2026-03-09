# SYSTEM GUARANTEES AND INVARIANTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ADVANCED_DAG_SEMANTICS.md
# Advanced DAG semantics and graph intelligence

## Typed semantics

The graph contract includes explicit typed semantics for:

- conditional execution
- branch decision nodes
- join reconciliation
- partition/map/reduce expansion
- window boundaries
- template expansion and graph composition

These semantics are represented as typed contracts, not runtime-only branching behavior.

## Deterministic expansion and normalization

- Dynamic edge expansion is allowed only when deterministic and snapshot-captured.
- Partitioned and expanded graphs are normalized to canonical graph form before execution planning.

## Parameter binding and late binding

- Graph-level, node-level, and runtime-level binding scopes are explicit.
- Late binding after compile is rejected when it would break snapshot immutability.

## Semantic diff and compatibility classification

Semantic diff reports classify changes as:

- topology
- policy
- metadata-only

Compatibility outcomes include:

- safe
- replay-safe
- cache-breaking
- schedule-breaking
- policy-breaking

## Static analysis and explainability

Graph intelligence includes:

- unreachable node detection
- dead branch detection
- no-op join detection
- explainability model for node/edge/order existence
- complexity scoring for governance and linting

## SOURCE: docs/spec/ADVERSARIAL_SYSTEM_RESILIENCE_CONTRACT.md
# Adversarial System Resilience Contract

## Purpose

This contract defines adversarial and stress expectations for `bijux-dag`
runtime, replay, cache, storage, and operator surfaces.

## Adversarial Coverage Classes

- adversarial DAG generation
- scheduler stress and starvation resistance
- artifact store stress and corruption resistance
- replay mismatch adversarial detection
- backend communication adversarial handling
- bundle import adversarial validation
- run history corruption resistance
- provenance traversal adversarial stability
- diff and explain adversarial robustness
- cache poisoning resistance
- environment drift adversarial detection
- adversarial concurrency behavior
- adversarial filesystem behavior
- determinism drift adversarial detection
- adversarial runtime crash recovery
- adversarial data corruption handling
- adversarial fuzzing and resilience verification

## Determinism and Safety

- Adversarial outcomes are reproducible under fixed seeds.
- Corruption and poisoning paths fail safely and diagnostically.
- Stress paths preserve invariant and telemetry visibility.


## SOURCE: docs/spec/ARCHITECTURE_REVIEW_CHECKLIST.md
# Architecture review checklist

- runtime module taxonomy is explicit and current
- sacred execution flow is documented and code-aligned
- crate boundaries match dependency policy
- naming policy is enforced for normative surfaces
- artifact storage and verification contracts are versioned
- control-plane command taxonomy is complete and enforced
- anti-drift and foundation suites are registered and owned
- unresolved architectural inconsistencies are explicitly listed

## SOURCE: docs/spec/BIJUX_SHARED_IDENTITY_CONTRACT.md
# Bijux Shared Identity Contract

## Purpose
Define identity surfaces shared across `bijux-cli`, `bijux-dag`, `bijux-atlas`, and `bijux-dna`.

## Shared identities
- graph identity (`graph_id` canonical hash)
- run identity (`run_id` with ancestry semantics)
- artifact identity (`artifact_id` content + provenance)

## Rules
- product adapters may extend metadata but cannot redefine shared identity semantics.
- cross-product import/replay must preserve shared identities or emit explicit downgrade.

## SOURCE: docs/spec/COMPARISON_HARNESS_CONTRACT.md
# Comparison Harness Contract

## Purpose
Comparisons are for evidence-driven analysis of:
Canonical terms are defined in `docs/spec/EVIDENCE_GLOSSARY.md`.
- correctness behavior
- operator ergonomics
- performance shape
- observability shape

Comparisons are not marketing claims.

## Initial external subset
- Dagster
- Prefect
- Argo Workflows

## Canonical scenarios
- chain
- diamond
- retry-timeout
- cache-reuse-shape
- replay-equivalence
- failure-propagation
- determinism
- operator-inspectability
- failure-diagnostics
- scheduler-tiny-tasks-overhead
- artifact-inspectability

## Comparable vs non-comparable
- Comparable:
  - terminal outcomes
  - retry/failure propagation classes
  - timeline and inspect surfaces
  - relative scheduler overhead trends under same scenario shape
- Not comparable:
  - absolute wall time across different host/container setups
  - feature areas one engine does not support natively
  - claims outside committed scenario scope

## Evidence policy
- Public comparison statements must cite committed harness artifacts in `evidence/compare/`.
- Interpretations must be separated from raw facts.
- Claims using “better”, “faster”, or “superior” require scenario-scoped evidence references.

## SOURCE: docs/spec/COMPARISON_METHOD_CONTRACT.md
# Comparison Method Contract

## Purpose
Define method rules for comparing benchmark outputs and publishing regressions.

## Comparison inputs
- Current benchmark report
- Baseline benchmark report
- Maximum allowed regression ratio threshold

## Method rules
- Compare only matching scenario IDs and benchmark classes.
- Missing baseline rows are reported as `unscored` and cannot justify performance claims.
- Ratio comparisons must use the same unit family (`ms`, `us`, bytes, throughput values).
- Threshold interpretation must be explicit: pass, warn, or fail.

## Command surface
Primary command: `cargo run -p bijux-dev-dag -- benchmark-compare --current <path> --baseline <path> --max-regression-ratio <value>`

## Output requirements
Comparison output must include:
- scenario ID
- benchmark class
- baseline value
- current value
- ratio
- threshold
- status

## SOURCE: docs/spec/CONCURRENCY_MODEL.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/CONCURRENCY_MODEL.md](./appendices/runtime/CONCURRENCY_MODEL.md)

## SOURCE: docs/spec/DAG_BEHAVIOR_DECISIONS.md
# DAG behavior decisions

## nondeterminism_allowed
`nondeterminism_allowed` is a transitional compatibility flag.

Permitted when enabled:
- Retry behavior for nodes that use clock/network effects without explicit deterministic seed input.

Never permitted even when enabled:
- Skipping validation of graph shape and reference integrity.
- Accepting invalid output paths.
- Accepting invalid node/tag/graph identifiers.

## group semantics
`group` is annotation only in v0.1.
It is not scheduling input and is excluded from node fingerprints.

## Node inputs and edge references
`inputs: Vec<String>` remains explicit and non-redundant in v0.1.
It defines node input interface contracts; edges bind dataflow to that interface.

## container payload
`container` remains a node-kind-specific payload in v0.1.
A typed cross-kind payload enum is deferred until a v0.2 compatibility window, to avoid breaking adapter contracts during boundary refactoring.

## SOURCE: docs/spec/DAG_MODEL_COMPLETENESS_CONTRACT.md
# DAG Model Completeness Contract

## Purpose

This contract defines complete DAG model semantics for validation, ordering,
node and dependency behavior, artifact dependencies, and normalization.

## Formal DAG Model Domains

- node semantic constraints
- dependency semantic constraints
- artifact dependency semantics
- node input/output contract semantics
- execution ordering guarantees
- DAG validation completeness guarantees
- DAG normalization determinism guarantees
- DAG schema compliance guarantees

## Verification Expectations

- DAG model compliance tests cover allowed and invalid shapes.
- semantic validation failures are explicit and deterministic.
- normalization output is deterministic for equivalent DAG inputs.
- schema checks and semantic checks are both required.
- semantic drift and anomaly checks are part of the verification suite.

## Tooling Surface

- `dag.lint`
- `dag.simulate`
- `dag.dry-run`
- `dag.plan-dump`
- `dag.explain-validation`
- `dag.schema-export`


## SOURCE: docs/spec/DAG_SPEC_V0.1.md
# DAG Spec v0.1

Source of truth schema: `configs/schema/dag.schema.json`.

## Overview
A DAG is a JSON document with `spec`, `nodes`, and `edges`. Nodes define typed operations and named ports. Edges connect output ports to input ports.

## Graph
```
{
  "spec": "bijux-dag/v0.1",
  "meta": { "name": "...", "description": "...", "owners": ["..."], "tags": ["..."] },
  "inputs": { "key": "value" },
  "nodes": [Node],
  "edges": [Edge]
}
```

## Node
```
{
  "id": "string",
  "kind": "const|shell|container|<external>",
  "inputs": ["string"],
  "outputs": [{"name":"file", "path":"relative/path"}],
  "params": <ParamValue>,
  "container": <ContainerSpec>,
  "effects": ["filesystem|network|env"],
  "env_allowlist": ["ENV_VAR"],
  "group": "etl/load"
}
```
- `id` must be unique and match `[a-zA-Z0-9_-]+`.
- `kind` determines executor behavior. External adapters use a custom string kind.
- `inputs` and `outputs` list valid ports. Outputs declare concrete files (paths are relative and must not include `..` or absolute paths).
- `params` is executor-specific.
- `effects` is required for `shell` nodes.
- `env_allowlist` lists env vars allowed for `shell` nodes.
- `container` is required for `container` nodes.
- `group` is organizational and not part of fingerprints.

### const params
```
{"value": <json>}
```

### shell params
```
{"argv": ["cmd", "arg1", ...]}
```

## ContainerSpec
```
{
  "image": "string",
  "argv": ["string"],
  "env_allowlist": ["ENV_VAR"],
  "workdir": "/bijux/node/work",
  "engine": "docker|podman"
}
```

## ParamValue
Params can be literal JSON values or explicit references. No string templating is allowed in v0.1.

```
<ParamValue> =
  <literal json>
| {"graph_input": "name"}
| {"node_output": {"node_id": "id", "path": "output_port"}}
| {"key": <ParamValue>, ...}
| [<ParamValue>, ...]
```

## Edge
```
{
  "from": {"node_id": "string", "port": "string"},
  "to": {"node_id": "string", "port": "string"}
}
```
Edges map a named output port to a named input port. Only the referenced output file is materialized for downstream nodes, under `nodes/<id>/inputs/<upstream>/<input_name>`.

## PortRef
```
{"node_id": "string", "port": "string"}
```

## Strictness
- Unknown fields are rejected.
- Missing required fields are rejected.
- Output paths are rejected at parse/validation boundary if absolute or containing `..`.
- Canonical naming grammar applies to graph name, node id, and tags.

## Canonicalization
- Nodes are sorted by `id`.
- Edges are sorted by (`from.node_id`, `from.port`, `to.node_id`, `to.port`).
- `inputs` and `outputs` are sorted.
- `params` object keys are sorted recursively.

## SOURCE: docs/spec/DEV_GOVERNANCE_ALLOWED_DEPENDENCIES.md
# Dev Governance Allowed Dependencies

This document defines dependencies for governance tooling in `bijux-dev-dag`.

## `bijux-dev-dag` allowed direct dependency classes

- workspace crates used for contracts and reports:
  - `bijux-dag-core`
  - `bijux-dag-runtime`
  - `bijux-dag-artifacts`
- governance/tooling crates:
  - `clap`
  - `serde`
  - `serde_json`
  - `sha2`
  - `hex`
  - `tempfile`

## Disallowed direct dependencies

- app routing crate: `bijux-dag-app`

## Enforcement

- `crates/bijux-dev-dag/tests/crate_taxonomy_guardrails.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`

## SOURCE: docs/spec/FINGERPRINTS_V0.1.md
# Fingerprints v0.1

`graph_id` is the canonical graph fingerprint identifier.

## Graph Fingerprint
- Compute canonical JSON for the entire graph.
- Hash with SHA256 of the UTF-8 bytes.
- Graph metadata is included when present.
- Exposed as `graph_id`.

Contributes directly:
- `spec`
- `meta.name`, `meta.description`, `meta.owners`, `meta.tags`
- `inputs`
- `nondeterminism_allowed`
- node list (after canonical ordering)
- edge list (after canonical ordering)

## Node Fingerprint
- Compute canonical JSON for the node only.
- Use resolved params (graph inputs substituted; node output refs resolve to declared output paths).
- Hash with SHA256 of the UTF-8 bytes.

Contributes directly:
- `id`
- `kind`
- `inputs`
- `outputs` (with canonical path normalization)
- resolved `params`
- `container`
- `timeout_ms`
- `resources`
- `tags`
- `retry`
- `effects`
- `env_allowlist`

## Runtime Node Fingerprint
- Start with the base node fingerprint above.
- Incorporate the materialized inputs index (path + sha256 + provenance) in a stable order.
- Hash with SHA256 of the UTF-8 bytes.

## Exclusions
- Runtime-only fields such as timestamps or execution status are excluded.
- `group` is excluded from node fingerprints.

## Explain surface

- `dag fingerprint --explain`
- `dag hash graph --explain`

## SOURCE: docs/spec/GRAPH_IDENTITY_CONTRACT.md
# Graph Identity Contract

## Graph identity

- `graph_id` is the canonical graph fingerprint.
- `graph_id` is derived as SHA256 over canonical graph JSON bytes.
- Canonicalization normalizes ordering and relative path separators.

## Implementation linkage

- `GraphId` type: `crates/bijux-dag-core/src/lib.rs`.
- Canonicalization entrypoints: `crates/bijux-dag-core/src/graph/canonical.rs`.
- Fingerprint entrypoints: `crates/bijux-dag-core/src/analysis/fingerprint.rs`.
- Topology ordering semantics: `crates/bijux-dag-core/src/graph/topology.rs`.

## Identity-affecting fields

- `spec`
- `meta.*` fields
- `inputs`
- `nondeterminism_allowed`
- all node semantics (kind, params, resources, env, outputs, effects, retry, timeout)
- edges

## Identity-non-affecting fields

- object key order in input JSON
- text formatting and line endings in source file
- node `group` (explicitly excluded from node fingerprint)

## Explain output

- `dag fingerprint --explain`
- `dag hash graph --explain`
- `dag canonical-diff` (machine-readable raw vs canonical diff)

Schema: `configs/schema/graph_fingerprint_explain.schema.json`
Schema: `configs/schema/graph_canonical_diff.schema.json`

## SOURCE: docs/spec/GRAPH_IDENTITY_FIELD_IMPACT.md
# Graph Identity Field Impact

This mapping documents which graph fields affect identity hashing.

## Included in graph identity

- `spec` after alias normalization (`0.1`/`v0.1` -> `bijux-dag/v0.1`)
- `inputs` (with map key sorting)
- `nondeterminism_allowed`
- node fields: `id`, `kind`, `inputs`, `outputs`, `params`, `container`, `timeout_ms`, `resources`, `retry`, `effects`, `env_allowlist`, `tags`, `group`
- edge fields: `from.node_id`, `from.port`, `to.node_id`, `to.port`

## Normalized before hashing

- node order (sorted by `id`)
- edge order (sorted by `from/to` tuple)
- `outputs.path` path separators
- `params` object key order
- `inputs` map key order
- `env_allowlist`, `effects`, `inputs`, `tags` ordering
- `resources` with `{cpu:0, mem_mb:0}` are normalized to `null`

## Excluded from graph identity

- backend adapter/runtime version metadata
- run-level metadata
- artifact-level metadata

## Generated report

Machine-readable decomposition:

- `docs/reports/foundation/graph_identity_decomposition_report.json`

## SOURCE: docs/spec/GRAPH_INPUT_READING_RESPONSIBILITIES.md
# Graph Input Reading Responsibilities

## Scope
Defines app-layer ownership for loading graph input before command execution.

## Rules
1. Filesystem input reading is owned by `crates/bijux-dag-app/src/read/fs_input.rs`.
2. Graph parse and spec-compat normalization are owned by `crates/bijux-dag-app/src/read/read_graph.rs`.
3. Command entry routing in `crates/bijux-dag-app/src/lib.rs` must delegate to these readers and must not duplicate graph-load parsing logic.
4. Graph read failures must exit before runtime execution side effects.

## SOURCE: docs/spec/IMPORT_EXPORT_CONTRACT.md
# Import Export Contract

## Scope
Defines export bundle formats, metadata-only behavior, and compatibility expectations.

## Invariants
- File-including export and metadata-only export have explicit, documented semantic differences.
- Import validates bundle structure before accepting artifacts.
- Import validates bundle version before accepting artifacts.

## Export modes
- `dag export --manifest-only`: exports manifest/snapshot/traces/output indexes without payload files.
- `dag export --with-files`: exports bundle including output file payloads.
- `dag export --without-artifacts`: exports manifest/snapshot/traces only, with empty outputs and no file payloads.
- `dag export --provenance-only`: exports provenance-focused evidence with empty traces and outputs maps.
- `dag export --redact`: redacts sensitive provenance fields during export.
- `dag export --manifest-only` and `dag export --with-files` are mutually exclusive.
- `dag export --from-run <path>` is an explicit source selector equivalent to positional `<run-dir>`.

## Bundle shape
- Required fields: `bundle_version`, `export_mode`, `manifest`, `graph_snapshot`, `node_traces`, `outputs`.
- `export_mode=manifest-only` requires `files` to be absent or `null`.
- `export_mode=with-files` requires `files` map payload.
- `export_mode=without-artifacts` requires `outputs` to be an empty map and `files` to be absent or `null`.
- `export_mode=provenance-only` requires both `node_traces` and `outputs` to be empty maps.
- `provenance.source` identifies source class (`native-run` today).

## Import verification mode
- `dag import --verify-only` performs version + invariant checks and returns summary output.
- `--verify-only` does not mutate run history state.
- `dag fsck <bundle.json>` provides bundle invariant verification via the fsck surface.

## Portability fidelity reporting
- Import summary includes `fidelity.level` (`exact` or `graded`) and `fidelity.downgrade_reasons`.

## Bundle versioning
- Current supported bundle version: `export-bundle/v0.1`.
- Unsupported versions must fail with explicit remediation.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`
- `evidence/compat/export_bundle/v0_1_supported/bundle.json`
- `evidence/compat/export_bundle/unsupported_past/bundle.json`

## Related schemas
- `configs/schema/run_manifest.schema.json`
- `configs/schema/outputs_index.schema.json`
- `configs/schema/operator/run_verify_report.schema.json`

## Versioning and change policy
Format changes require compatibility fixtures for supported windows.

## SOURCE: docs/spec/LARGE_DAG_SCALABILITY_CONTRACT.md
# Large DAG Scalability Contract

## Purpose

Define expected behavior and quality signals for large DAG execution and analysis workloads.

## Required workload coverage

- DAG size: 1,000 nodes
- DAG size: 10,000 nodes
- large fan-out structures
- large fan-in structures
- deep dependency chains

## Required execution surfaces

- planner scalability
- scheduler scalability
- runtime memory under large DAG load
- artifact generation under large DAG load
- replay planning under large DAG load
- diff performance under large DAG load
- provenance traversal under large DAG load
- explain behavior under large DAG load

## Required governance artifacts

- huge DAG stress fixture corpus
- large run history stress corpus
- artifact store stress corpus for large runs
- runtime profiling and telemetry summaries for large DAG workloads
- DAG memory footprint regression benchmarks
- scalability regression suite

## SOURCE: docs/spec/NODE_FINGERPRINT_FIELD_IMPACT.md
# Node Fingerprint Field Impact

This mapping documents node-level fields that contribute to node fingerprinting.

## Included in node fingerprint

- `id`
- `kind`
- `inputs` (sorted)
- `outputs.name`
- `outputs.path` (path-normalized)
- `params` (object-key normalized)
- `container`
- `timeout_ms`
- `resources` (normalized defaults)
- `retry`
- `effects` (sorted)
- `env_allowlist` (sorted)
- `tags` (sorted)
- `group`

## Excluded from node fingerprint

- adapter runtime metadata
- run/provenance metadata
- artifact storage metadata

## SOURCE: docs/spec/NODE_IDENTITY_CONTRACT.md
# Node Identity Contract

## Scope

Defines stable node identity semantics for graph authoring, planning, execution traces,
and artifact lineage.

## Canonical node identifier

- Canonical field: `node.id` (string) in DAG documents.
- `node.id` must be unique within a graph.
- `node.id` is stable across planning and runtime traces.

## Implementation linkage

- Graph model: `crates/bijux-dag-core/src/lib.rs` (`Node` and `PortRef` structures).
- Topology and edge linkage: `crates/bijux-dag-core/src/graph/topology.rs` and `crates/bijux-dag-core/src/graph/edge.rs`.
- Validation guards: `crates/bijux-dag-core/src/pipeline/validate.rs`.

## Identity invariants

- Edges reference nodes through `from.node_id` and `to.node_id`.
- Planner lowering preserves `node.id` as execution-plan node identity.
- Runtime trace and outputs index preserve node identity linkage.

## Relationship to fingerprints

- `node.id` is one of the direct contributors to node fingerprints.
- Node fingerprints are deterministic over canonical node semantics.
- Graph identity is derived from canonical graph bytes and includes node semantics and edges.

## Related contracts

- `docs/spec/GRAPH_IDENTITY_CONTRACT.md`
- `docs/spec/FINGERPRINTS_V0.1.md`
- `docs/spec/PLANNER_CONTRACT.md`
- `docs/spec/RUN_ARTIFACT_SPEC_V0.1.md`

## SOURCE: docs/spec/OBSERVABILITY_CONTRACT.md
# Observability Contract

## Scope

Defines operator-facing observability guarantees for runtime runs.

## Layers

- logs: structured event records
- metrics: typed run/node/scheduler metrics
- traces: timeline and attempt-level state transitions

## Required Runtime Events

Required event names:

- `run_started`
- `node_ready`
- `node_scheduled`
- `node_started`
- `node_attempt_started`
- `node_attempt_finished`
- `node_failed`
- `run_finished`

Required event fields:

- `name`
- `unix_ms`
- `run_id`
- `category`

## Required Metrics

Scheduler metrics:

- queue depth
- ready count
- running count
- completed count
- retry count
- cache hits
- cache misses
- failure count
- dispatch latency
- concurrency pressure

Run metrics:

- makespan
- success ratio
- parallelism utilization
- cache reuse ratio
- artifact volume
- planning duration
- scheduling wait duration
- execution duration
- trace write duration
- manifest finalize duration
- replay compare duration

## Timeline and Debug Artifacts

- `observability.timeline.json` is required for completed and failed runs.
- `observability.events.json` is required for completed and failed runs.
- `observability.root-causes.json` is required for failed runs.

## Secret Redaction

Observability payloads must not include raw secret/token/password values in
public runtime event details.

## Contract Checks

- Event name and required field checks are enforced in runtime tests.
- Control-plane suite `observability-contract` validates docs/test alignment.

## Operator vs Developer Surfaces

Operator and developer observability surface split is tracked in
`docs/tracking/OBSERVABILITY_SURFACE_PLAN.md`.

## SOURCE: docs/spec/PLANNER_ANALYSIS.md
# Planner analysis and optimization contracts

## Planner phase model

Planner execution is represented as explicit phases:

1. normalize
2. validate
3. bind
4. optimize
5. schedule-ready transform

## Selection, replay, and backfill planning

- Node annotations capture why a node is selected, deferred, skipped, or replayed.
- Replay plans distinguish execute and skip actions.
- Partial-run closure expansion is deterministic.
- Backfill plans include explicit window boundaries and partition keys.

## Resource and placement intelligence

- Planner estimates aggregate CPU and memory requirements from node contracts.
- Priority inheritance is derived from graph and node policy hints.
- Locality and queue placement hints are emitted as plan annotations.

## Compatibility and guardrails

- Planner validates backend capability compatibility before execution.
- Impossible runs are rejected at plan time (for example invalid resource contracts).
- Optimizer rules may only alter behavior when guardrails explicitly permit semantic optimization.

## Fingerprints, diffs, and explainability

- Plan fingerprints are stable identities for equivalent plan outputs.
- Plan diffs capture order/filter/annotation changes.
- Explain-plan output summarizes phases, annotations, and optimization notes.

## Benchmark fixture set

- `evidence/perf/fixtures/planner_large_fanout.json`
- `evidence/perf/fixtures/planner_deep_chain.json`
- `evidence/perf/fixtures/planner_mixed_resources.json`

## SOURCE: docs/spec/PLANNER_CONTRACT.md
# Planner contract

## Authority

This document is the single normative authority for planner inputs, lowering stages, outputs, and guarantees.

## Boundary model

- parsed graph: strict parse output
- validated graph: parse output after schema and semantic validation
- canonical graph: deterministic normalization for identity and ordering
- execution plan: lowered runtime representation

## Lowered plan model

Execution plan uses lowered structures only:

- `PlannedNode`: execution-relevant fields (`id`, `kind`, `deps`, `outputs`, `retry`, `timeout_ms`)
- `PlannedEdge`: lowered dependency edge (`from`, `to`)

Planner boundary owns graph lowering; runtime execution consumes lowered plan semantics.

## Fingerprints

- `graph_fingerprint`: canonical graph identity
- `planner_fingerprint`: lowered plan identity

These fingerprints have distinct meaning and must not be conflated.

## Determinism guarantees

- semantically equivalent graphs lower to identical planner fingerprints
- cosmetic metadata changes do not alter lowered semantic plan identity
- plan ordering is deterministic

## Validation and diagnostics

- schema/semantic validation errors are distinct from planner lowering errors
- planner diagnostics use stable IDs:
  - `P4000`: planner generic failure
  - `P4013`: unsupported node kind
  - `P4016`: warning for outputless execution node
  - `P4021`: runtime capability requirement rejected during lowering

## Selector and pruning stage

Selector pruning occurs after validation and before final lowering output is committed.

## Graph shape coverage

Planner lowering coverage includes:

- fan-in graphs
- fan-out graphs
- disconnected graphs (supported)

## Debug and schema surfaces

- debug command: `bijux-dev-dag dag plan-dump --graph <path>`
- schema: `configs/schema/execution_plan.schema.json`

## Required evidence

- `crates/bijux-dag-core/tests/planner_contract.rs`
- `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`
- `crates/bijux-dev-dag/tests/planner_hardening_contracts.rs`
- `planner-alignment` control-plane suite

## Trust property linkage

Planner guarantees are tracked under battle trust property `tp_plan_truth`.

## SOURCE: docs/spec/PORTABILITY_GUARANTEES.md
# Portability Guarantees

## Exact guarantees

- Bundle schema validation for supported versions.
- Canonical graph identity preservation when graph snapshot is present.
- Import invariant checks for required structural fields.

## Fidelity-graded guarantees

- Provenance completeness when source context is redacted.
- Artifact replay equivalence when payloads are omitted (`without-artifacts`, `provenance-only`).
- Cross-environment reproducibility with backend capability drift.

## Interpretation

Import summaries expose `fidelity.level` and `fidelity.downgrade_reasons` for machine-readable portability status.

## SOURCE: docs/spec/PROVENANCE_MODEL_CONTRACT.md
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

## SOURCE: docs/spec/SACRED_EXECUTION_FLOW.md
# Sacred Execution Flow

## Canonical pipeline

The runtime sacred flow is:

1. plan
2. schedule
3. execute
4. collect
5. persist
6. advance

Expanded checkpoint sequence:

1. validate graph and contracts
2. lower graph to execution plan
3. initialize `ExecutionContext` and scheduler state
4. compute dependency readiness
5. materialize declared node inputs
6. compute node fingerprint and cache lookup
7. execute adapter with centralized retry logic
8. collect node result and classify terminal status
9. write trace and attempt events
10. propagate failure/skip/cached outcomes deterministically
11. write cache on eligible success paths
12. finalize manifest and artifact indexes
13. advance run state to terminal outcome

## Canonical context and result models

- Run-scoped context: `execution_context::ExecutionContext`
- Node-scoped context: `execution_context::NodeExecutionContext`
- Canonical node result: `node_result::NodeResult`

## Sacred centralized hooks

- retry logic: `sacred_execution::run_retry_logic`
- failure propagation: engine policy branch handling
- artifact materialization: `sacred_execution::run_materialize_inputs`
- cache read/write: `sacred_execution::run_cache_lookup` / `sacred_execution::run_cache_write`
- readiness/dependency: `sacred_execution::resolve_dependencies` and `ready_queue_from_dependencies`
- trace writing: `sacred_execution::run_write_trace`

## Side-channel execution prohibition

- Runtime node execution must not bypass sacred hooks for retry, cache, trace, and dependency readiness.
- Direct cache/trace wiring in engine code is forbidden when a sacred hook exists.

## Failure-injection expectations

- Sacred flow checkpoints have failure-injection tests proving deterministic failure handling.
- Failure-injection evidence is tracked in runtime sacred-flow contract tests and foundation hardening reports.

## State machine guards

- run transitions use `state_machine::run_transition_allowed`
- node transitions use `state_machine::node_transition_allowed`
- invariant and verify surfaces reject illegal terminal accounting

## Replay path contract

Replay must call `Runtime::run` over replay snapshot graph and therefore share the same engine path and state transition rules.

## SOURCE: docs/spec/SPEC_CONTRACT_FAMILY_INVENTORY_41_60.md
# Specification contract family inventory (tasks 41-60)

## Terminology (7)
- NAMING.md
- NAMING_GUIDELINES.md
- TERMS.md
- NAMING_REVIEW_POLICY.md
- NAMING_PHILOSOPHY.md
- GLOSSARY.md
- TERMINOLOGY_GLOSSARY.md

## VersioningAndSchema (11)
- SCHEMA_COMPATIBILITY_POLICY.md
- SCHEMA_FIELD_DEPRECATION_POLICY.md
- SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md
- VERSION_COMPATIBILITY_DRIFT_POLICY.md
- SCHEMA_EVOLUTION_RULEBOOK.md
- VERSIONING_MODEL.md
- SCHEMA_FORWARD_COMPATIBILITY_LIMITATIONS.md
- UNIFIED_SCHEMA_VERSIONING_POLICY.md
- SCHEMA_EVOLUTION_POLICY.md
- VERSIONING.md
- BUNDLE_MANIFEST_VERSIONING_POLICY.md

## Config (5)
- CONFIG_STATE_BOUNDARIES.md
- CONFIG_PRECEDENCE.md
- CONFIG_CONTRACT.md
- CONFIG_PRECEDENCE_CONTRACT.md
- CONFIG_DEPRECATION.md

## CLI (6)
- CLI_COMMAND_STABILITY_DOCUMENTATION.md
- CLI_BACKWARD_COMPATIBILITY.md
- CLI_SURFACE_STABILITY_POLICY.md
- CLI_OWNERSHIP.md
- CLI_CONTRACT.md
- CLI_DEPRECATION_AND_ALIAS_POLICY.md

## BackendAdapter (12)
- BACKEND_PROTOCOL_STABILITY_CONTRACT.md
- ADAPTER_PLACEMENT.md
- BACKEND_EXECUTION_MATURITY.md
- BACKEND_CONTRACT.md
- ADAPTER_CONTRACT.md
- ADAPTER_RUNTIME_CONTRACT_V0.1.md
- ADAPTER_INTERFACE_SPEC_V0.1.md
- BACKEND_MEANING_BOUNDARY_DOCTRINE.md
- BACKEND_EQUIVALENCE_CONTRACT.md
- HPC_ADAPTER_CONTRACT.md
- ATLAS_EXECUTION_CONTRACT.md
- K8S_ADAPTER_CONTRACT.md

## Benchmark (7)
- BENCHMARK_SCORECARD_GUIDE.md
- BENCHMARK_TYPES.md
- BENCHMARK_MINIMALISM_POLICY.md
- BENCHMARK_RAW_DATA_RETENTION.md
- BENCHMARK_SCENARIO_CONTRACT.md
- BENCHMARK_RESULT_FORMAT.md
- BENCHMARK_REPRODUCIBILITY_CONTRACT.md

## Evidence (7)
- EVIDENCE_INTERNAL_ONLY_SURFACES.md
- EVIDENCE_TERMS_AND_GOVERNANCE.md
- EVIDENCE_PUBLICATION_CONTRACT.md
- EVIDENCE_RELEASE_NOTE_TRUST.md
- EVIDENCE_GLOSSARY.md
- AUDIT_REPORT_CONTRACT.md
- EVIDENCE_MODEL.md

## Graph (10)
- NODE_TRACE_SCHEMA_V0.1.md
- NODE_STATE_MACHINE.md
- NODE_IDENTITY_CONTRACT.md
- GRAPH_DIFF_SPEC_V0.1.md
- NODE_FINGERPRINT_FIELD_IMPACT.md
- GRAPH_INPUT_READING_RESPONSIBILITIES.md
- GRAPH_IDENTITY_CONTRACT.md
- GRAPH_DIFF_SEMANTICS.md
- GRAPH_BUNDLE_FORMAT_V1.md
- GRAPH_IDENTITY_FIELD_IMPACT.md

## Artifact (15)
- ARTIFACT_INSPECT_SCHEMA_V0.1.md
- ARTIFACT_DIFF_SEMANTICS.md
- ARTIFACT_IDENTITY_PROVENANCE_MAPPING.md
- ARTIFACT_DURABILITY_GUARANTEES_CONTRACT.md
- ARTIFACT_SYSTEM.md
- ARTIFACT_PLATFORM.md
- ARTIFACT_RETENTION_POLICY.md
- ARTIFACT_LINEAGE_COMPLETENESS_CONTRACT.md
- ARTIFACT_IDENTITY_CONTRACT.md
- ARTIFACT_STORAGE_LIFECYCLE_CONTRACT.md
- ARTIFACT_BUNDLE_FORMAT_V1.md
- ARTIFACT_LIFECYCLE.md
- ARTIFACT_OWNERSHIP_TABLE.md
- ARTIFACT_INTEGRITY_SUITE.md
- ARTIFACT_BUNDLE_MANIFEST_EXAMPLES.md

## ReplayDiff (8)
- REPLAY_EQUIVALENCE_COMPLETENESS_CONTRACT.md
- REPLAY_FIDELITY_LEVELS.md
- RUN_DIFF_SPEC_V0.1.md
- DIFF_CLASSIFICATION_CONTRACT.md
- RUN_DIFF_SEMANTICS.md
- REPLAY_EVIDENCE_CONTRACT.md
- REPLAY_PROOF_BUNDLE_SCHEMA_V0.1.md
- REPLAY_CONTRACT.md

## RunRuntime (45)
- EXECUTION_TRACE_RECORDS_CONTRACT.md
- RUNTIME_ARCHITECTURE_CLEANUP_CONTRACT.md
- RUN_MANIFEST_SCHEMA_V0.1.md
- RUNTIME_FAULT_TOLERANCE_CONTRACT.md
- RUN_DIR_CONTRACT.md
- RUN_DIRECTORY_FILESYSTEM_GUARANTEES.md
- RUNTIME_SEMANTICS_CONTRACT.md
- RUNTIME_OVERREACH_REDUCTION_POLICY.md
- CACHE_SYSTEM_INTEGRITY_CONTRACT.md
- CACHE_CONTRACT.md
- EXECUTION_ENGINE_CONTRACT.md
- OUTPUTS_INDEX_SCHEMA_V0.1.md
- RUN_BUNDLE_FORMAT_V1.md
- SCHEDULER_WORKLOAD_MANAGEMENT.md
- EXECUTION_SEMANTICS_CONTRACT.md
- DISTRIBUTED_EXECUTION_ARCHITECTURE_CONTRACT.md
- RUNTIME_PUBLIC_API_BOUNDARY.md
- RUN_DIR_EVOLUTION_RULEBOOK.md
- RUN_DIR_STORAGE_CONTRACT.md
- RUN_HISTORY_CONTRACT.md
- RUNTIME_TELEMETRY_SCHEMA.md
- OBSERVABILITY_CONTRACT.md
- SCHEDULER_CONTRACT.md
- EXECUTION_ACCEPTANCE_GATES.md
- SCHEDULER_STATE_TRANSITIONS.md
- CACHE_SEMANTICS.md
- SCHEDULER_FAIRNESS_DETERMINISM.md
- RUN_DIR_OWNERSHIP.md
- RUN_HISTORY_CORRUPTION_RECOVERY.md
- OUTPUT_CONCISION_CONTRACT.md
- CACHE_EVOLUTION_MODEL.md
- RUNTIME_ALLOWED_DEPENDENCIES.md
- FAILURE_TAXONOMY_CONTRACT.md
- RUNTIME_SCOPE_GOVERNANCE_POLICY.md
- RUN_MANIFEST_EVOLUTION_MATRIX.md
- RUN_VS_ARTIFACT_LINEAGE.md
- RUN_RECOVERY_AND_RESILIENCE.md
- CACHE_PRUNE_POLICY.md
- RUN_STATE_MACHINE.md
- RUN_ARTIFACT_SPEC_V0.1.md
- RUN_IDENTITY_CONTRACT.md
- RUN_SUMMARY_SCHEMA_V0.1.md
- SCHEDULER_STATESPACE_CONTRACT.md
- EXECUTION_KERNEL_DETERMINISM_GUARANTEES.md
- DISTRIBUTED_COORDINATION_MODEL.md

## System (8)
- SYSTEM_CONCEPTUAL_INTEGRITY_CONTRACT.md
- SYSTEM_MAINTAINABILITY_CONTRACT.md
- SYSTEM_RELIABILITY_GUARANTEES_CONTRACT.md
- SYSTEM_INTROSPECTION_ARCHITECTURE_CONTRACT.md
- SYSTEM_INTROSPECTION_COMMANDS_CONTRACT.md
- SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md
- SYSTEM_COMPLETENESS_VERIFICATION_CONTRACT.md
- SYSTEM_FORMAL_INVARIANTS_CONTRACT.md

## GovernanceOrGuidance (22)
- SPEC_TO_CODE_AND_TEST_OWNERSHIP.md
- DEV_GOVERNANCE_ALLOWED_DEPENDENCIES.md
- VOCABULARY_SCOPE_HONESTY_POLICY.md
- FOUNDATION_READINESS_CRITERIA.md
- PLACEHOLDER_SURFACE_POLICY.md
- DOCS_GOVERNANCE.md
- TRUTH_BEFORE_CONVENIENCE_DOCTRINE.md
- CANONICAL_FIXTURE_STRATEGY_POLICY.md
- MODELED_AND_FUTURE_SURFACES.md
- RELEASE_POLICY.md
- ANTI_DRIFT_POLICY.md
- PATH_NORMALIZATION_POLICY.md
- MIGRATION_POLICY.md
- AS_UNDERSCORE_IMPORT_POLICY.md
- HISTORY_RETENTION_POLICY.md
- INTERNAL_CONTRACT_DISCIPLINE_POLICY.md
- FEATURE_DEVELOPMENT_FREEZE_POLICY.md
- KERNEL_DEPENDENCY_POLICY.md
- MISSION_STATEMENT.md
- FIXTURE_TOOLING_GOVERNANCE_CONTRACT.md
- CURRENT_IMPLEMENTED_CAPABILITIES.md
- CRATE_API_POLICY.md

## Other (106)
- CRATE_TAXONOMY_V2.md
- REFERENCE_RUNTIME.md
- LARGE_DAG_SCALABILITY_CONTRACT.md
- ARCHITECTURE_REVIEW_CHECKLIST.md
- CONTROL_PLANE_FOUNDATION.md
- OPERATOR_INSPECTION_CONTRACT.md
- RESOURCE_PROFILE_STRATEGY.md
- RELEASE_BINARY_VERIFICATION.md
- PROVENANCE_MODEL_CONTRACT.md
- DETERMINISM.md
- EXTENSION_CATALOG_CONTRACTS.md
- STABLE_SCHEMA_COMPATIBILITY_REVIEW_CHECKLIST.md
- REPOSITORY_STRUCTURAL_HEALTH_CONTRACT.md
- PLANNER_CONTRACT.md
- DAG_SPEC_V0.1.md
- PORTABILITY_GUARANTEES.md
- VALIDATION_RULES.md
- ANALYTICS_EXACTNESS.md
- ERROR_CONTRACT.md
- COMPATIBILITY_PROMISE.md
- REMOTE_DELIVERY_GUARANTEES.md
- SEMANTIC_DIFF_COMPLETENESS_CONTRACT.md
- TESTKIT_EVIDENCE_ACCESS_CONTRACT.md
- REMOTE_EXECUTION_MODEL.md
- ADVANCED_EXPLAINABILITY_MODEL_CONTRACT.md
- CRATE_RESPONSIBILITY_ALIGNMENT.md
- ATTEMPT_TRACE_SCHEMA_V0.1.md
- OPERATOR_UX_CONTRACT.md
- RELEASE_ARTIFACTS.md
- ADVANCED_SEMANTICS_SCOPE.md
- RELEASE_REVIEW_CHECKLIST.md
- CONTROL_PLANE_COMMAND_TAXONOMY.md
- CRATE_OWNERSHIP.md
- PERFORMANCE_STRATEGY.md
- ADVANCED_DAG_SEMANTICS.md
- ADVANCED_SEMANTICS_RETAINED_SURFACES.md
- BATTLE_TRUST_PROPERTIES.md
- AUTHORING_UX_CONTRACT.md
- BOUNDARY_RULES.md
- SANDBOX_SECURITY_MODEL_CONTRACT.md
- STATE_MACHINE_VISUALIZATION.md
- API_CONTRACT.md
- SACRED_EXECUTION_FLOW.md
- ENVIRONMENT_IDENTITY_CONTRACT.md
- ERROR_TAXONOMY.md
- TEST_TRUST_LEDGER.md
- POLICY_CONTRACT.md
- WORKER_PROTOCOL_CONTRACT.md
- EXTENSIBILITY_CONTRACT.md
- TEST_STRATEGY.md
- INTERNAL_INVARIANTS_CONSISTENCY_CONTRACT.md
- CRATE_OWNERSHIP_MATRIX.md
- README.md
- ROOT_MESSAGING_CONTRACT.md
- WORKSPACE_CONTRACT.md
- DIAGNOSTICS_MODES.md
- SELECTOR_CONTRACT.md
- EXPLAIN_SURFACES_CONTRACT.md
- DAG_BEHAVIOR_DECISIONS.md
- CONTAINER_EXECUTION_CONTRACT.md
- DETERMINISTIC_SCHEDULING_CONTRACT.md
- INTEGRITY_EVIDENCE_CONTRACT.md
- SPEC_CONTRACT_FAMILY_INVENTORY_41_60.md
- TRACE_CONTRACT.md
- DAG_MODEL_COMPLETENESS_CONTRACT.md
- ADVANCED_SEMANTICS_QUARANTINED_SURFACES.md
- PROJECT_CONTRACT.md
- DETERMINISM_EVIDENCE_CONTRACT.md
- BIJUX_SHARED_IDENTITY_CONTRACT.md
- COMPARISON_METHOD_CONTRACT.md
- PERFORMANCE_OPTIMIZATION_CONTRACT.md
- ADVERSARIAL_SYSTEM_RESILIENCE_CONTRACT.md
- BUNDLE_SCHEMA_REFERENCE.md
- TEST_EVIDENCE_CONSUMER_CONTRACT.md
- CRATE_RESPONSIBILITY_STATEMENTS.md
- TASK_CONTRACT_TYPES.md
- BATTLE_WORKFLOW_CONTRACT.md
- KERNEL_BOUNDARY_CONTRACT.md
- IMPORT_EXPORT_CONTRACT.md
- KERNEL_ALLOWED_DEPENDENCIES.md
- SECURITY_MODEL.md
- POLICY_EVALUATION_TRACE.md
- CRATE_BOUNDARY_CONTRACT.md
- ERROR_CODES.md
- OPERATOR_UX_CHECKLIST.md
- FORMAL_INVARIANTS.md
- COMPARISON_HARNESS_CONTRACT.md
- EXPORT_BUNDLE_EVOLUTION_RULEBOOK.md
- TEST_TRUST_CONTRACT.md
- FINGERPRINTS_V0.1.md
- STATE_MACHINE_CONTRACT.md
- CONCURRENCY_MODEL.md
- BIJUX_CLI_INTEGRATION_CONTRACT.md
- ADOPTION_SURFACES.md
- BATCH_EXECUTION_MODEL.md
- MULTI_RUN_ANALYTICS_CONTRACT.md
- STORAGE_CONTRACT.md
- CPU_MEMORY_BUDGET_MODEL.md
- CANONICAL_GRAPH_IDENTITY_SPEC.md
- PROOF_BUNDLE_CONTRACT.md
- WORK_STEALING_SCHEDULING_BOUNDARIES.md
- PLANNER_ANALYSIS.md
- DNA_EXECUTION_CONTRACT.md
- TEST_PHILOSOPHY.md
- PERFORMANCE_CONTRACT.md
- STABLE_EXPERIMENTAL_SCHEMA_FIELDS.md


## SOURCE: docs/spec/SYSTEM_COMPLETENESS_VERIFICATION_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_COMPLETENESS_VERIFICATION_CONTRACT.md](./appendices/system/SYSTEM_COMPLETENESS_VERIFICATION_CONTRACT.md)

## SOURCE: docs/spec/SYSTEM_CONCEPTUAL_INTEGRITY_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_CONCEPTUAL_INTEGRITY_CONTRACT.md](./appendices/system/SYSTEM_CONCEPTUAL_INTEGRITY_CONTRACT.md)

## SOURCE: docs/spec/SYSTEM_INTROSPECTION_ARCHITECTURE_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_INTROSPECTION_ARCHITECTURE_CONTRACT.md](./appendices/system/SYSTEM_INTROSPECTION_ARCHITECTURE_CONTRACT.md)

## SOURCE: docs/spec/SYSTEM_INTROSPECTION_COMMANDS_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_INTROSPECTION_COMMANDS_CONTRACT.md](./appendices/system/SYSTEM_INTROSPECTION_COMMANDS_CONTRACT.md)

## SOURCE: docs/spec/SYSTEM_MAINTAINABILITY_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_MAINTAINABILITY_CONTRACT.md](./appendices/system/SYSTEM_MAINTAINABILITY_CONTRACT.md)

## SOURCE: docs/spec/SYSTEM_RELIABILITY_GUARANTEES_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_RELIABILITY_GUARANTEES_CONTRACT.md](./appendices/system/SYSTEM_RELIABILITY_GUARANTEES_CONTRACT.md)

## SOURCE: docs/spec/TASK_CONTRACT_TYPES.md
# Task contract type system

## Type vocabulary

Task contracts use a formal type registry:

- scalar types
- collection types
- versioned serialization rules
- explicit coercion rules

Silent coercion is forbidden. Coercion must be declared and compatibility-scoped.

## Contract semantics

- nullability, optionality, cardinality
- secret references (distinct from normal strings/env)
- resource references for datasets, models, or prior artifacts
- partitioned collection contracts
- bounded polymorphic variants

## Adapter and replay compatibility

- Adapter capability declarations are checked against type requirements.
- Replay compatibility checks validate adapter version alignment and declared support.

## Validation and diagnostics

- Parameter default compatibility validation
- Cross-node producer/consumer contract validation
- Path-level diagnostics for mismatch locations

## Fingerprints and evolution

- Task contract fingerprints are separate from node fingerprints.
- Output evolution markers include backward and forward compatibility flags.

## Generated documentation and matrix reporting

- Contract markdown can be generated directly from typed contract structures.
- Compatibility matrix reports summarize producer-consumer relationships across a DAG snapshot.

## SOURCE: docs/spec/appendices/runtime/CONCURRENCY_MODEL.md
# Concurrency Model

## Scope

This document defines concurrency guarantees for runtime scheduling, execution
coordination, artifact writes, cache access, and run finalization.

## Ownership Model

- Scheduler readiness state is owned by `SchedulerState`.
- Execution orchestration state is owned by runtime engine loop.
- Shared cross-worker counters and status maps are owned by runtime
  coordination primitives.
- Storage path mutation is centralized under run-dir and cache APIs.

## Shared Mutable State Inventory

| State | Owner | Synchronization | Notes |
| --- | --- | --- | --- |
| ready queue and indegree | `SchedulerState` | single-owner mutable state | deterministic updates |
| retry queue | `SchedulerState` | single-owner mutable state | explicit retry requeue |
| scheduler event log | `SchedulerState` | single-owner mutable state | ordered sequence IDs |
| run summary counters | runtime coordination | `Mutex` | monotonic count updates |
| trace write records | runtime coordination | `Mutex` | atomic append semantics |
| cache claim map | runtime coordination | `Mutex` | single fingerprint claim |
| latest-link update lock | runtime coordination | `Mutex` | prevents concurrent mutation races |

## Scheduling Concurrency Guarantees

- Concurrent predecessor completion can unlock a downstream node at most once.
- Retry requeue cannot duplicate node eligibility.
- Cancellation and timeout are terminal for scheduling decisions in a loop tick.
- Concurrency level tuning (`jobs`, `max_parallelism`) may alter throughput but
  not semantic node-set outcomes for deterministic plans.

## Artifact and Cache Concurrency Guarantees

- Trace append operations are serialized by coordination locks.
- Cache claims for a fingerprint are single-winner per in-memory coordination
  instance.
- Run summary updates are monotonic and merged under one lock.

## Unsafe Policy

- Runtime crate policy: no `unsafe` unless documented by an ADR and covered by
  dedicated tests.
- Control-plane audit reports every `unsafe` block and owner file.

## Stress and Flake Discipline

- Deterministic stress tests run medium graphs repeatedly under high
  concurrency.
- Any nondeterministic failure must be recorded in the concurrency flake ledger.

## Recovery and In-Progress Access

- Import/export against in-progress run directories is rejected unless explicitly
  supported with a contract update.
- Controller restart recovery semantics must be explicit; if unsupported, fail
  fast with a clear diagnostic.

## Verification Surfaces

- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `crates/bijux-dag-runtime/tests/concurrency_contracts.rs`
- `bijux-dev-dag repo run --domain governance` suites:
- `scheduler-invariants`
- `runtime-unsafe-audit`
- `concurrency-model`

## SOURCE: docs/spec/appendices/system/SYSTEM_COMPLETENESS_VERIFICATION_CONTRACT.md
# System Completeness Verification Contract

## Purpose

This contract defines the final completeness review expectations across
invariants, determinism, replay, diff, lineage, runtime/scheduler, adapters,
portability, schema compatibility, introspection, observability, security,
performance, stress, fuzz, conceptual integrity, architecture coherence,
correctness guarantees, and reliability guarantees.

## Completeness Review Expectations

- all completion-contract domains remain present and machine-readable
- cross-domain coverage remains linked through verification suites
- final completeness report summarizes verification scope and status
- no domain is considered complete without explicit contract/test/report anchors

## Final Verification Scope

- coverage review: invariants, determinism, replay, diff, lineage, runtime
- platform review: adapters, portability, schema compatibility, introspection
- operational review: observability, security, performance, stress, fuzz
- integrity review: conceptual integrity, architecture coherence, correctness,
  reliability


## SOURCE: docs/spec/appendices/system/SYSTEM_CONCEPTUAL_INTEGRITY_CONTRACT.md
# System Conceptual Integrity Contract

## Purpose

This contract defines conceptual integrity expectations for architecture,
execution model, artifact model, replay model, diff model, provenance model,
backend abstraction, runtime behavior, scheduler behavior, and determinism
claims.

## Conceptual Model Surfaces

- conceptual architecture overview
- system execution model overview
- system artifact model overview
- system replay model overview
- system diff model overview
- system provenance model overview
- backend abstraction model overview
- runtime execution model overview
- scheduler behavior model overview
- system determinism guarantees overview

## Integrity Expectations

- model narratives remain consistent across docs and command surfaces
- architecture conformance checks remain explicit and deterministic
- conceptual drift detection surfaces remain active
- verification tooling and suites remain machine-readable


## SOURCE: docs/spec/appendices/system/SYSTEM_INTROSPECTION_ARCHITECTURE_CONTRACT.md
# System Introspection Architecture Contract

## Purpose

This contract defines architecture-level guarantees for system introspection
surfaces, data consistency, determinism, reliability, and diagnostics.

## Architecture Guarantees

- introspection commands expose stable operator-visible semantics
- introspection data remains internally consistent across command surfaces
- introspection outputs are deterministic for equal inputs
- introspection behavior under failure is explicit and diagnosable
- introspection performance and telemetry surfaces are continuously verifiable

## Verification Expectations

- command correctness tests
- JSON schema stability tests
- determinism tests
- failure-path behavior tests
- regression fixtures
- performance benchmarks
- anomaly detection tests
- telemetry reporting tests
- diagnostics tooling checks
- visualization data checks
- fuzz and stress coverage
- reliability tests
- architecture review and verification suite


## SOURCE: docs/spec/appendices/system/SYSTEM_INTROSPECTION_COMMANDS_CONTRACT.md
# System Introspection Commands Contract

## Purpose

This contract defines durable expectations for operator-facing introspection in
`bijux-dag`. Introspection commands must provide deterministic, machine-readable
diagnostics without mutating run state.

## Command Surface

The introspection surface is composed of command entrypoints and their backing
handlers.

- `dag.run-inspect` -> `run_dag_run_inspect`
- `dag.scheduler-timeline` -> `run_dag_scheduler_timeline`
- `storage-health` -> `run_storage_health`
- `backend-registry-report` -> `run_backend_registry_report`
- `cache-coverage-report` -> `run_cache_coverage_report`
- `verify.evidence-replay` -> `run_evidence_replay_verify`
- `drift-dashboard` -> `run_drift_dashboard`
- `verify.evidence-drift` -> `run_evidence_drift_verify`

## Determinism Rules

- JSON object keys and list ordering are deterministic for equal inputs.
- Text output ordering is deterministic for equal inputs.
- Introspection commands must never depend on wall-clock ordering for stable
  report fields.

## Integrity Rules

- Commands detect malformed or missing metadata and report clear anomalies.
- Commands must return parseable JSON when `--json` is requested.
- Introspection evidence files must be schema-validated where schemas exist.

## Coverage Rules

Introspection verification must cover:

- execution trace inspection
- artifact store health inspection
- run history integrity inspection
- scheduler state inspection
- backend capability inspection
- replay compatibility inspection
- cache state inspection
- provenance graph inspection
- artifact lineage graph inspection
- runtime diagnostics inspection
- deterministic output ordering
- schema validation
- snapshot stability
- telemetry and anomaly reporting
- stress behavior

## Non-goals

- release distribution workflows
- runtime mutation or repair semantics


## SOURCE: docs/spec/appendices/system/SYSTEM_MAINTAINABILITY_CONTRACT.md
# System Maintainability Contract

## Purpose

This contract defines long-term maintainability expectations for repository
structure, ownership, boundaries, layering, dependency hygiene, and operational
verification.

## Maintainability Rules

- module ownership rules are explicit and enforced
- module boundary policies are explicit and enforced
- architectural layering rules are explicit and enforced
- dependency hygiene and cycle checks are explicit and enforced
- complexity and drift monitoring are tracked and reported

## Verification Expectations

- architectural boundary tests
- dependency cycle detection tests
- module complexity monitoring tests
- maintainability regression fixtures
- maintainability telemetry and anomaly checks
- maintainability conformance tests
- maintainability review and verification tooling
- maintainability verification suite


## SOURCE: docs/spec/appendices/system/SYSTEM_RELIABILITY_GUARANTEES_CONTRACT.md
# System Reliability Guarantees Contract

## Purpose

This contract defines reliability guarantees and verification expectations for
runtime, artifacts, replay, scheduler behavior, and operational diagnostics.

## Reliability Targets

- runtime reliability targets
- artifact durability and reliability targets
- replay reliability targets
- scheduler reliability targets

## Verification Expectations

- reliability target tests
- reliability regression fixtures
- reliability stress suite
- reliability telemetry and anomaly reporting
- runtime/artifact/replay reliability benchmarks
- reliability diagnostics tooling
- reliability failure simulation and monitoring tests
- reliability architecture review and verification suite

