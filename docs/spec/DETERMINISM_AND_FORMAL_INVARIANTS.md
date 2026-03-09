# DETERMINISM AND FORMAL INVARIANTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/DETERMINISM.md
# Determinism

## What Must Be Stable
- Canonical JSON output for a DAG.
- Fingerprints for graphs and nodes.
- Execution order for ready nodes (stable topo ordering).
- Artifact layout and file naming.

## What May Vary
- Actual wall-clock timestamps.
- Runtime performance and scheduling latency.

## Forbidden
- Non-deterministic scheduling that changes trace ordering.
- Reading undeclared environment variables.
- Hidden runtime-only fields that affect fingerprints.

## SOURCE: docs/spec/DETERMINISTIC_SCHEDULING_CONTRACT.md
# Deterministic scheduling contract

For deterministic workloads, scheduling outcomes must be invariant across worker parallelism values.

## Scope
Defines deterministic scheduling behavior for planning, dispatch ordering, failure propagation, and retry accounting.

Contract requirements:
- `jobs=1` and `jobs>1` produce equivalent manifests and outputs for deterministic DAGs.
- Ready-node tie breaking is stable for equal priority nodes.
- Failure propagation decisions are deterministic from graph state and selection state.
- Retry backoff metadata is persisted and replay-explainable.

## Related tests
- `crates/bijux-dag-runtime/tests/scheduler_determinism.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Versioning and change policy
Any scheduling semantic change must update deterministic fixtures and the scheduler contract tests before merge.

## SOURCE: docs/spec/EXECUTION_KERNEL_DETERMINISM_GUARANTEES.md
# Execution Kernel Determinism Guarantees

## Purpose

Define deterministic behavior guarantees for graph execution, planning, replay, and diagnostics.

## Guaranteed deterministic surfaces

- run results for identical graph, inputs, and environment
- node ordering for identical DAG topology
- scheduler outcomes for identical readiness and priority inputs
- artifact hash values for identical artifact bytes
- diff output ordering
- replay planning ordering
- provenance traversal ordering
- explain output ordering
- CLI JSON key ordering and stable envelopes

## Required robustness checks

- fuzz checks for DAG structure variation
- fuzz checks for environment variation
- fuzz checks for artifact path ordering
- fuzz checks for scheduling tie-break behavior
- fuzz checks for runtime event ordering
- regression fixtures for determinism drift
- failure detection for deterministic mismatch
- telemetry and trend reporting for deterministic drift

## Release expectations

- determinism regressions are release blocking for stable surfaces
- drift reports must be generated from current fixture corpus

## SOURCE: docs/spec/FORMAL_INVARIANTS.md
# Formal Invariants

## Invariant registry

| ID | Domain | Invariant | Enforcement |
| --- | --- | --- | --- |
| INV-GRAPH-SHAPE-001 | graph | graph is acyclic and references are valid | graph validation + generated-shape tests |
| INV-GRAPH-SHAPE-002 | graph | node ids are unique | graph validation |
| INV-GRAPH-SHAPE-003 | graph | canonical order is stable | formal invariant property tests |
| INV-PLAN-SHAPE-001 | plan | executable plan contains deterministic dependency structure | planner tests + property tests |
| INV-SCHED-READY-001 | scheduler | downstream node becomes ready exactly once | scheduler contract tests |
| INV-SCHED-STATE-001 | scheduler | terminal node state does not revert | scheduler contract tests |
| INV-RUN-COUNTS-001 | run_state | manifest totals match node trace terminal statuses | runtime invariant check + `dag runs verify` |
| INV-RUN-TERMINAL-001 | run_state | completed run includes at least one terminal node status | `dag runs verify` |
| INV-TRACE-TIME-001 | trace | trace finished time is not before start time | `dag runs verify --deep` |
| INV-TRACE-ATTEMPT-001 | trace | trace attempt metadata is coherent for a node | trace schema + trace tests |
| INV-CACHE-PROOF-001 | cache | cache hit requires compatible proof metadata | cache evolution tests |
| INV-ARTIFACT-REF-001 | artifacts | indexed artifact entries point to existing files | `dag runs verify` |
| INV-ARTIFACT-PATH-001 | artifacts | indexed artifact paths are normalized relative paths | `dag runs verify --deep` |
| INV-EXPORT-VERIFY-001 | import_export | imported and exported bundles pass invariant checks before use | import/export contract tests |
| INV-REPLAY-EQUIV-001 | replay | replay comparison equivalence reasons are explicit and deterministic | replay diff tests |

## Coverage rule
- Every invariant ID must map to at least one code check, test, or control-plane guard.
- New normative guarantees must reference invariant IDs directly.

## Enforcement command
- `bijux-dev-dag invariants-report` prints the current registry and mapped coverage.

## Guarantee wording rule
Claims using words such as “guarantee”, “always”, or “never” in normative docs must cite at least one invariant ID in `INV-*` format.

## SOURCE: docs/spec/INTERNAL_INVARIANTS_CONSISTENCY_CONTRACT.md
# Internal Invariants Consistency Contract

## Purpose

Define required internal invariant guarantees for graph, planner, runtime, scheduler, artifact store, and run history state.

## Required invariant coverage

- invariant assertions for graph, planner, runtime, scheduler, artifact store, and run history state
- invariant violation detection and failure logging behavior
- invariant monitoring telemetry and anomaly detection
- invariant regression fixtures, stress, and fuzz verification
- invariant debugging tooling and trace-capture behavior
- invariant performance impact benchmarking

## Required governance artifacts

- internal invariants regression corpus
- internal invariants verification suite
- invariant telemetry report
- invariant debugging report
- invariant coverage report
- invariant performance impact report

## SOURCE: docs/spec/SCHEDULER_FAIRNESS_DETERMINISM.md
# Scheduler Fairness and Determinism

The runtime scheduler is deterministic by default:

- ready queue tie-break uses lexicographic node id ordering
- dispatch decisions are reproducible for identical inputs and budgets

Fairness tradeoff:

- deterministic mode prioritizes repeatability and replay explainability
- throughput mode prioritizes queue drain rate while preserving contract invariants


## SOURCE: docs/spec/SYSTEM_FORMAL_INVARIANTS_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_FORMAL_INVARIANTS_CONTRACT.md](./appendices/system/SYSTEM_FORMAL_INVARIANTS_CONTRACT.md)

## SOURCE: docs/spec/SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md
# System guarantees and invariants contract

**What this spec is not**: roadmap speculation, implementation internals, or detailed architecture guidance.

## Scope

Canonical cluster for system-level reliability, formal invariants, introspection, and completeness.

- reliability and operational target guarantees
- invariant and completeness expectations
- introspection architecture and command surfaces
- health diagnostics policy
- maintainability expectations affecting system shape

## Consolidated rules

- Reliability and correctness guarantees are explicit and cross-referenced in tests and evidence.
- Invariant drift is detectable through deterministic suites and drift dashboards.
- Introspection and diagnostics remain non-mutating unless explicitly scoped.
- Completeness reporting remains tied to measurable coverage checks.

## Implementation and evidence links

- Core implementations: `crates/bijux-dag-*`, `docs/architecture`, `docs/reference`
- Validation sources: governance suites and completion/invariant contracts in `crates/bijux-dev-dag`

## SOURCE: docs/spec/appendices/system/SYSTEM_FORMAL_INVARIANTS_CONTRACT.md
# System Formal Invariants Contract

## Purpose

This contract defines system-level invariants that must hold across DAG
evaluation, replay, diff, identity, backend equivalence, and import/export
operations.

## System Invariant Domains

- core DAG execution invariants
- artifact lineage invariants
- replay equivalence invariants
- diff semantic invariants
- scheduler fairness invariants
- run identity invariants
- artifact identity invariants
- backend equivalence invariants
- determinism invariants

## Verification Expectations

- invariants are checked for successful runs
- invariants are checked for failed runs
- invariants are checked for partial runs
- invariants are checked during replay flows
- invariants are checked during import/export flows
- invariant failures are explicitly logged
- invariant drift is detected through deterministic corpus checks

## Operator Surface

- `invariants-report` is the canonical command surface for invariant status.
- invariant verification artifacts are machine-readable and versioned.

