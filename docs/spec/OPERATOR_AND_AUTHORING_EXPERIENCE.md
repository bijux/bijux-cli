# OPERATOR AND AUTHORING EXPERIENCE

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ADVANCED_EXPLAINABILITY_MODEL_CONTRACT.md
# Advanced Explainability Model Contract

## Purpose

Define the advanced explainability model for deterministic, complete, and operator-actionable explanation outputs across execution, replay, cache, lineage, and backend capability surfaces.

## Required explanation dimensions

- node-level execution explanations
- scheduler decision explanations
- replay decision explanations
- cache hit and cache miss explanations
- artifact lineage and dependency chain explanations
- environment drift explanations
- backend capability mismatch explanations

## Required quality guarantees

- consistent explain output across repeated inspections
- stable JSON schema and text snapshot surfaces
- deterministic explain ordering
- explain completeness verification for partial and anomalous conditions
- explain stress behavior under large DAG workloads

## Required governance artifacts

- advanced explainability regression corpus
- advanced explainability stress verification suite
- explainability performance benchmark report
- explainability anomaly and completeness reports
- explainability coverage report

## SOURCE: docs/spec/ANALYTICS_EXACTNESS.md
# Analytics Exactness Model

## Exact analytics
- `runs compare` field-by-field values copied from run summaries.
- `runs failures` counts derived from recorded trace failure kinds.
- `runs summary` totals (run count, retries, cache hits, artifact counts).

## Heuristic analytics
- `runs flakes` as status divergence grouped by graph fingerprint.
- trend interpretation over incomplete history.
- determinism and replay signals inferred from observed retries and outcomes.

## Interpretation boundary
Heuristic outputs are indicators for investigation and are not formal correctness proofs.

## SOURCE: docs/spec/AUTHORING_UX_CONTRACT.md
# Authoring UX Contract

## Supported authoring surface
`bijux-dag` currently supports one authoritative authoring surface: JSON DAG files that conform to `spec: "0.1"`.

YAML/DSL/generated authoring are not normative product surfaces in this repository.

## Canonical examples
- Minimal executable DAG: `evidence/authoring/patterns/minimal.json`
- Medium executable DAG with retries/resources/selectors: `evidence/authoring/patterns/medium.json`

## Authoring evidence classification
- `minimal`: first-hour onboarding baseline.
- `patterns`: normative reusable graph structures.
- `negative`: normative invalid inputs bound to stable validation rule IDs.
- `examples`: illustrative end-to-end authoring samples.

Battle workflows under `evidence/battle/` are not allowed to be reused as authoring fixtures.

## Pattern examples
- chain: `evidence/authoring/patterns/pattern_chain.json`
- diamond: `evidence/authoring/patterns/pattern_diamond.json`
- fanout: `evidence/authoring/patterns/pattern_fanout.json`
- aggregation: `evidence/authoring/patterns/pattern_aggregation.json`
- cache-heavy: `evidence/authoring/patterns/pattern_cache_heavy.json`
- replay-sensitive: `evidence/authoring/patterns/pattern_replay_sensitive.json`

## Common mistake fixtures
- undeclared outputs: `evidence/authoring/negative/undeclared_outputs.json`
- invalid refs: `evidence/authoring/negative/invalid_refs.json`
- cycles: `evidence/authoring/negative/cycle.json`
- invalid selectors: `evidence/authoring/negative/invalid_selectors.json`
- unsupported adapter payload: `evidence/authoring/negative/unsupported_adapter_payload.json`

## Authoring command surfaces
- `dag validate --explain <dag>`
- `dag graph-lint <dag>`
- `dag canonicalize <dag>`
- `dag show-effective-graph <dag>`
- `dag show-effective-plan <dag>`

## Naming guidance
- Node IDs must be unique, stable, and domain-specific.
- IDs must avoid ambiguous short aliases.
- Edge references must target existing node IDs and declared ports.
- Guidance is tied to validation rules documented in `docs/spec/VALIDATION_RULES.md`.

## Documentation and fixture rule
Examples in user-facing docs must reference executable fixture files under `evidence/authoring/`.
Hand-maintained prose-only DAG snippets are not allowed as normative examples.

## Intentionally out of scope
See `docs/user/AUTHORING_GUIDE.md` section `What this DAG tool intentionally does not do`.

## SOURCE: docs/spec/EXPLAIN_SURFACES_CONTRACT.md
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

## SOURCE: docs/spec/OPERATOR_INSPECTION_CONTRACT.md
# Operator inspection contract

## Scope
Defines stable operator inspection surfaces, output semantics, and failure classification behavior for run inspection commands.

## Stable commands

- `dag runs inspect`
- `dag runs show`
- `dag runs timeline`
- `dag runs tree`
- `dag runs explain-failure`
- `dag runs verify`
- `dag runs doctor`

## Integrity classification

Inspection outputs must classify run integrity using:

- `healthy`
- `incomplete`
- `corrupt`
- `unsupported`

## JSON schema surfaces

- `configs/schema/operator/run_inspect.schema.json`
- `configs/schema/operator/run_show.schema.json`
- `configs/schema/operator/run_timeline.schema.json`
- `configs/schema/operator/run_tree.schema.json`
- `configs/schema/operator/run_explain_failure.schema.json`
- `configs/schema/operator/run_doctor.schema.json`

## Human-readable surfaces

- `dag runs show` must provide a concise summary focused on run identity, status, integrity, and timing.
- `dag runs inspect` must remain concise while including retries, cache hits, and artifact counts.

## Timeline reconstruction requirements

Timeline reconstruction must include:

- execution ordering by start timestamp
- retry attempt information
- cache-hit markers
- coherence with trace timestamps

## Portability and context rules

- Inspection commands must work from explicit run roots without ambient repository state.
- Imported runs must remain inspectable and distinguishable in integrity/provenance reporting.

## Versioning and change policy
Any inspection output shape change requires schema updates, contract updates, and operator tests in the same change.

## SOURCE: docs/spec/OPERATOR_UX_CHECKLIST.md
# Operator UX Checklist

This checklist defines the minimum operator-facing quality bar for `bijux-dag-app` command surfaces.

## Scope

- Validate output remains concise and remediation-oriented.
- Plan output explains inclusion/exclusion decisions deterministically.
- Run output always reports the created run directory.
- Inspect and history output remain stable for automation and humans.
- Replay and diff output communicate equivalence status clearly.
- Prove and verify output expose integrity/completeness state directly.
- Artifact inspect output includes identity, provenance, and lineage fields.

## Contract Links

- App service boundary report: `docs/reports/foundation/app_service_boundary_report.md`
- Operator UX contract: `docs/spec/OPERATOR_UX_CONTRACT.md`
- Output concision contract: `docs/spec/OUTPUT_CONCISION_CONTRACT.md`

## Test Coverage Links

- Human snapshots: `crates/bijux-dag-app/tests/operator_human_snapshot_contracts.rs`
- Schema lockstep checks: `crates/bijux-dag-app/tests/operator_schema_lockstep_contracts.rs`
- No-panic malformed input checks: `crates/bijux-dag-app/tests/operator_input_no_panic_contracts.rs`

## SOURCE: docs/spec/OPERATOR_UX_CONTRACT.md
# Operator UX Contract

## Personas
- local developer
- CI runner
- benchmark runner
- incident investigator
- release verifier

## Operator command classes
- run-time: `dag run`, `dag replay`
- inspect-time: `dag runs list|show|inspect|tree|timeline|diff|explain-failure|summary|compare|trend|failures|flakes`
- repair-time: `dag runs doctor`
- repo-time: `bijux-dev-dag repo run --domain governance`

## Stable operator run inspection surfaces
- `dag runs list --root <runs_dir>`
- `dag runs show <run_id> --root <runs_dir>`
- `dag runs inspect <run_id> --root <runs_dir>`
- `dag runs tree <run_id> --root <runs_dir>`
- `dag runs timeline <run_id> --root <runs_dir>`
- `dag runs diff <run_a_dir> <run_b_dir>`
- `dag runs verify <run_id> --root <runs_dir> [--deep]`
- `dag runs doctor <run_id> --root <runs_dir>`
- `dag runs explain-failure <run_id> --root <runs_dir>`
- `dag runs summary --root <runs_dir>`
- `dag runs compare <run_a> <run_b> --root <runs_dir>`
- `dag runs trend --root <runs_dir>`
- `dag runs failures --root <runs_dir>`
- `dag runs flakes --root <runs_dir>`

## Exit semantics
- `0`: command succeeded and reported healthy/valid result
- `3`: run data invalid/corrupt/missing for verify and doctor failure cases
- `2`: command usage/argument contract error
- `1`: internal error

## Output contracts
- Every run-inspection command supports `--json`.
- JSON schemas are in `configs/schema/operator/`.

## Corruption behavior
- inspection commands must return partial diagnostics when possible.
- verify and doctor must fail explicitly on invalid run state.

## Non-repo-coupled behavior
All `dag runs ...` commands operate on explicit `--root` and `run_id` inputs and
must not depend on ambient repository files.

## Command taxonomy ownership
Normative operator command taxonomy lives in:
- `docs/user/OPERATOR_WORKFLOWS.md`
- `docs/reference/COMMAND_TAXONOMY.md`

## Inspection contract ownership
- `docs/spec/OPERATOR_INSPECTION_CONTRACT.md`
- `docs/user/OPERATOR_WORKFLOWS.md`
