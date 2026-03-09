# RUN HISTORY AND RECOVERY

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/HISTORY_RETENTION_POLICY.md
# History Retention Policy

## Retention baseline
- Keep authoritative run directories under the configured runs root.
- Retention is currently manual; no automatic deletion is applied by analytics commands.

## Operational guidance
- Keep enough history for flake and trend analysis windows used by your team.
- Prune old runs with explicit operator action outside analytics commands.

## Authority model
- Authoritative run data: run directory contents produced by execution.
- Derived analytics caches: optional, disposable, and recomputable.

## Corruption handling
- Corrupt or partial runs remain part of history but are surfaced as degraded inputs.
- Analytics commands should report partial views rather than mutating or repairing history.

## SOURCE: docs/spec/MULTI_RUN_ANALYTICS_CONTRACT.md
# Multi-Run Analytics Contract

## Scope
Multi-run analytics are supported over an explicit runs root directory.
Commands are read-only and never mutate authoritative run records.

## Minimal analytics surfaces
- `dag runs summary --root <runs_dir>`
- `dag runs compare <run_a> <run_b> --root <runs_dir>`
- `dag runs trend --root <runs_dir>`
- `dag runs failures --root <runs_dir>`
- `dag runs flakes --root <runs_dir>`

## Run index model
- Run history is the set of direct child directories under `--root`.
- Each run directory is treated as authoritative local evidence.
- Analytics are derived views over that authoritative set.

## Incomplete history behavior
- Missing optional artifacts are tolerated where possible.
- Corrupt JSON is treated as unknown/null fields, not process crash.
- Commands keep returning partial aggregates when enough evidence exists.

## Aggregated output schema
JSON output for analytics commands must conform to:
- `configs/schema/operator/runs_analytics.schema.json`

## Determinism and replay signals
`dag runs summary` emits report sections:
- determinism report
- cache usefulness report
- replay equivalence report
- failure distribution report

These reports summarize observed history and do not assert stronger guarantees than the evidence supports.

## Data authority boundary
- Authoritative: run manifests, snapshots, traces, outputs indexes.
- Derived: analytics aggregates and trend series.
- Rule: analytics must never rewrite authoritative run files.

## SOURCE: docs/spec/RUN_BUNDLE_FORMAT_V1.md
# Run Bundle Format v1

## Identifier

`run-bundle/v1`

## Required fields

- `bundle_version`: `export-bundle/v0.1`
- `format`: `run-bundle/v1`
- `manifest`
- `graph_snapshot`
- `node_traces`
- `outputs`

## Optional fields

- `files`
- `provenance`

## Invariants

- Run bundle import must preserve run ancestry/provenance fields when present.
- `node_traces` keys must match `node_id` inside each trace payload.

## SOURCE: docs/spec/RUN_DIFF_SEMANTICS.md
# Run Diff Semantics

Run diff compares:

- manifest contract fields
- graph fingerprint
- node outcomes and fingerprints
- output payload hashes

The replay equivalence report is the authoritative semantic result.

## SOURCE: docs/spec/RUN_DIFF_SPEC_V0.1.md
# Run Diff Spec v0.1

## Scope

Run diff compares two run directories using manifest, graph fingerprint, node traces, and output payload indexes.

## Inputs

- `manifest.json` for run A and run B
- `graph.snapshot.json` for run A and run B
- node trace payloads under `nodes/*/trace.json`
- output index payloads under `nodes/*/outputs/index.json`

## Semantic Dimensions

- `manifest`
- `graph_fingerprint`
- `nodes`
- `outputs`

## Required Output Fields

- `manifest` (object)
- `graph_fingerprint` (object or null)
- `nodes` (object)
- `outputs` (object)
- `replay_equivalence.equivalent` (boolean)
- `replay_equivalence.reasons` (array)
- `replay_equivalence.reason_report` (object)
- `replay_equivalence.cause_groups` (object)

## Cause Group Contract

- `manifest_drift`
- `graph_semantics`
- `node_outcomes`
- `artifact_payload`

## Determinism Requirements

- repeated execution on identical run inputs MUST produce byte-identical JSON output
- node and output keys MUST be ordered deterministically

## Non-Goals

- wall-clock performance attribution
- policy recommendation beyond reported causes

## SOURCE: docs/spec/RUN_DIRECTORY_FILESYSTEM_GUARANTEES.md
# Run Directory Filesystem Guarantees

## Purpose

Define required filesystem and run-directory behavior for durable run records.

## Required guarantees

- deterministic run directory layout and file naming
- deterministic artifact path generation
- deterministic metadata ordering for machine-readable files
- concurrency-safe run directory creation
- recovery behavior after crashes and partial writes
- repair behavior for partial and corrupted run directories
- migration compatibility for supported run-dir schema versions
- portability behavior across filesystem path conventions

## Integrity checks

- corrupted event log detection
- corrupted node metadata detection
- missing metadata recovery handling
- consistency verification for manifest, node traces, and output indices

## Safety checks

- filesystem permission handling
- filesystem race condition resistance
- atomic write guarantees for critical metadata files
- corruption stress and recovery benchmarking coverage

## SOURCE: docs/spec/RUN_DIR_CONTRACT.md
# Run Directory Contract

## Scope
Defines run directory layout, mandatory files, optional files, and compatibility rules.

## Required entries (authoritative)
- `manifest.json`
- `graph.snapshot.json`
- `nodes/<node_id>/trace.json`
- `outputs/index.json`

## Optional entries
- `latest` symlink
- `provenance.json`
- cache proof payloads attached to node traces

## Derived artifacts (non-authoritative)
- timeline and inspect reports reconstructed from authoritative artifacts
- analytics summaries
- comparison reports

## Verification behavior
- `dag verify` (standard): requires `manifest.json` and `outputs/index.json`.
- `dag verify --deep`: adds schema parsing checks.
- `dag verify --strict`: requires all authoritative entries and supported `manifest_version`.
- Missing optional entries must not fail standard verification.

## Ownership
- File ownership mapping is defined in `docs/spec/RUN_DIR_OWNERSHIP.md`.

## Invariants
- Paths are relative, normalized, and non-escaping.
- Historical runs are immutable after finalization.
- `latest` link updates must not mutate historical run payloads.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`

## Related schemas
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`
- `configs/schema/outputs_index.schema.json`
- `configs/schema/operator/run_verify_report.schema.json`

## Versioning and change policy
Additive optional files are allowed. Required structure changes require compatibility review and fixture migration.

## SOURCE: docs/spec/RUN_DIR_EVOLUTION_RULEBOOK.md
# Run Directory Evolution Rulebook

- `manifest_version` is the authoritative run-dir format tag.
- Required file removal is breaking.
- Required file additions require defaults/backfill strategy.
- Unsupported versions must fail verify/doctor with precise diagnostics.

## SOURCE: docs/spec/RUN_DIR_OWNERSHIP.md
# Run Directory Ownership

## Purpose
Define ownership for each persisted run artifact so write/read responsibility is explicit.

## Ownership table

| Artifact path | Authoritative | Owner module |
| --- | --- | --- |
| `manifest.json` | yes | `bijux-dag-artifacts::RunDir` |
| `graph.snapshot.json` | yes | `bijux-dag-artifacts::RunDir` |
| `nodes/<node_id>/trace.json` | yes | `bijux-dag-artifacts::RunDir` + runtime engine writer |
| `outputs/index.json` | yes | `bijux-dag-artifacts::RunDir` + runtime engine writer |
| `provenance.json` | no | app import/export surface |
| `latest` symlink | no | app run lifecycle surface |

## Rules
- Only owner modules may define path conventions for authoritative run artifacts.
- New authoritative files require contract update in `RUN_DIR_CONTRACT.md` and this table.
- Derived files must never override or shadow authoritative files.

## SOURCE: docs/spec/RUN_DIR_STORAGE_CONTRACT.md
# Run directory storage contract

## Scope

Defines canonical run directory structure, required files, and verification behavior.

## Required files

- `manifest.json`
- `outputs.index.json`
- `trace/`

## Finalization files

- `manifest.finalized.json`
- `.run-complete.json`

## Incomplete marker

- `.run-incomplete.json` is written when a run ends before finalization.

## Verification modes

- `standard`: required files and manifest parse checks.
- `strict`: standard checks plus `manifest_version` and finalization files.

## SOURCE: docs/spec/RUN_HISTORY_CONTRACT.md
# Run History Contract

## Scope

Defines machine-readable run ancestry and history query behavior.

## Command surfaces

- `dag runs history --root <runs_dir>`
- `dag runs id-explain <run_id> --root <runs_dir>`
- `dag runs summary --root <runs_dir>`
- `dag runs doctor <run_id> --root <runs_dir>`

## Schema surfaces

- `configs/schema/operator/run_history.schema.json`
- `configs/schema/operator/run_id_explain.schema.json`

## Invariants

- History output must include `run_id`, `parent_run_id`, and `source_run_id`.
- History queries are read-only and must never mutate run artifacts.
- Missing manifests produce actionable but non-panicking diagnostics.
- Missing trace surfaces referenced by manifest counters must be reported by doctor output.
- History traversal order is deterministic (`run_id` lexical order).
- `latest` alias updates are advisory and must not mutate historical rows.

## Ancestry field mapping

- `parent_run_id`: replay parent linkage from source run identity.
- `source_run_id`: origin run used for replay/import lineage.

## Recovery

- See `docs/spec/RUN_HISTORY_CORRUPTION_RECOVERY.md` for corruption handling and operator recovery procedure.

## SOURCE: docs/spec/RUN_HISTORY_CORRUPTION_RECOVERY.md
# Run History Corruption Recovery

## Scope

This note defines the current recovery behavior for corrupted run-history directories consumed by `dag runs history`, `dag runs summary`, and `dag runs id-explain`.

## Current behavior

- Directory traversal is authoritative: each directory under the selected `--root` is treated as a run candidate.
- Corrupt `manifest.json` files are tolerated without panic; history rows are still emitted with `null`/fallback values.
- Analytics queries must not mutate authoritative run records.
- Alias artifacts such as `latest` must not rewrite or reorder historical run rows.

## Operator recovery steps

1. Run `dag runs history --root <runs_dir>` to enumerate all recoverable rows.
2. Run `dag runs doctor <run_id> --root <runs_dir>` for suspicious entries.
3. Rebuild or replace only corrupt run directories; do not rewrite healthy run manifests.
4. Re-run `dag runs summary --root <runs_dir>` to confirm recovered aggregate state.

## Non-goals

- Automatic rewriting of corrupted manifests.
- Silent deletion of corrupted run directories.

## SOURCE: docs/spec/RUN_IDENTITY_CONTRACT.md
# Run Identity Contract

## Run identity

- `run_id` is the immutable identifier for a finalized run directory.
- `run_id` must remain stable for the lifetime of historical run artifacts.
- `run_id` explanation and ancestry surfaces are available through:
  - `dag runs id-explain <run_id> --root <runs_dir>`
  - `dag runs history --root <runs_dir>`
  - `dag runs show <run_id> --root <runs_dir>`

Implementation anchors:
- `crates/bijux-dag-app/src/inspect/run_views.rs` (`explain_run_id`, `runs_history`, `inspect_summary`)
- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs` (run manifest authorship and replay ancestry wiring)

## Composition

`run_id` is runtime-assigned and persisted in `manifest.json`. It is not derived from mutable alias links.

## Ancestry fields

- `run_metadata.parent_run_id`
- `run_metadata.source_run_id`

These fields must also appear in the JSON identity explanation output.

## Immutability

Historical run content must not be mutated by alias updates such as `--latest`.

## SOURCE: docs/spec/RUN_MANIFEST_EVOLUTION_MATRIX.md
# Run Manifest Evolution Matrix

| Version | Status | Required migration behavior |
| --- | --- | --- |
| `run-manifest/v0.1` | supported | parse and verify with strict required keys |
| pre-`v0.1` | unsupported | fail with compatibility diagnostics |

## Test matrix owner

- `crates/bijux-dev-dag/tests/run_manifest_evolution_contracts.rs`

## SOURCE: docs/spec/RUN_RECOVERY_AND_RESILIENCE.md
# Run recovery and resilience contracts

This contract defines run orchestration and recovery behavior for pause/resume, interruption handling, metadata repair, and fault simulation.

## Pause and operator control

- `RunPausePolicy` defines deterministic pause handling for queued, ready, and dispatch behavior.
- `NodeControlMode` supports explicit operator node pause/block records.
- Running nodes can be preserved while queue/dispatch is frozen based on `RunPauseMode`.

## Restart and scheduler recovery

- `PersistedRunSnapshotRef` captures persisted run snapshots used on process restart.
- `SchedulerRecoveryRule` maps orphaned node states to deterministic actions:
  - reattach
  - requeue
  - mark failed
  - quarantine

## Heartbeats, stuck detection, and interruption classes

- `NodeHeartbeatPolicy` and `StuckRunPolicy` define principled liveness thresholds.
- `InterruptionClass` distinguishes:
  - clean shutdown
  - process crash
  - worker loss
  - backend loss
- `ResumePolicy` defines explicit resume choices: reattach, verify, rerun incomplete, fail-safe stop.

## Operator retry, checkpoint resume, and branch-local recovery

- `OperatorRetryPolicy` captures audited manual retry constraints.
- `ManualInterventionRecord` stores first-class human interventions.
- `CheckpointResumeContract` records checkpoint-based node resume capability.
- `BranchRecoveryMode` enables either fail-fast or continue-healthy-branches execution.

## Degraded mode and resilience verification

- `DegradedExecutionPolicy` supports execution when optional services are unavailable.
- Recovery fault simulation contracts:
  - `RecoveryFaultInjection`
  - `RecoverySimulationScenario`
  - `RecoveryAcceptanceSuite`
- Fixture paths:
  - `evidence/perf/fixtures/recovery/power_loss_restart.json`
  - `evidence/perf/fixtures/recovery/acceptance_suite.json`

## Metadata repair, consistency checks, and quarantine

- `validate_and_repair_run_metadata` defines repair behavior for missing manifest/index files.
- `check_run_consistency` validates node state, artifact state, and run summary consistency.
- `RunQuarantineRecord` captures suspicious or inconsistent run evidence.

## CLI control-plane paths

`bijux-dev-dag` exposes:

- `dag repair-run --run-dir <path> [--apply]`
- `dag simulate-recovery --scenario <fixture.json>`
- `dag recovery-accept --suite <fixture.json>`

## SOURCE: docs/spec/RUN_STATE_MACHINE.md
# Run state machine

States:
- `queued`
- `ready`
- `running`
- `succeeded`
- `failed`
- `cached`
- `skipped`
- `cancelled`

Legal transitions:
- `queued -> ready`
- `ready -> running`
- `running -> succeeded|failed|cached|skipped|cancelled`
- `ready -> cancelled`
- `queued -> cancelled`

Illegal transitions are rejected by contract tests.
