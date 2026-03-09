# BENCHMARK AND PERFORMANCE EVIDENCE

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/BENCHMARK_MINIMALISM_POLICY.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_MINIMALISM_POLICY.md](./appendices/benchmark/BENCHMARK_MINIMALISM_POLICY.md)

## SOURCE: docs/spec/BENCHMARK_RAW_DATA_RETENTION.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_RAW_DATA_RETENTION.md](./appendices/benchmark/BENCHMARK_RAW_DATA_RETENTION.md)

## SOURCE: docs/spec/BENCHMARK_REPRODUCIBILITY_CONTRACT.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_REPRODUCIBILITY_CONTRACT.md](./appendices/benchmark/BENCHMARK_REPRODUCIBILITY_CONTRACT.md)

## SOURCE: docs/spec/BENCHMARK_RESULT_FORMAT.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_RESULT_FORMAT.md](./appendices/benchmark/BENCHMARK_RESULT_FORMAT.md)

## SOURCE: docs/spec/BENCHMARK_SCENARIO_CONTRACT.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_SCENARIO_CONTRACT.md](./appendices/benchmark/BENCHMARK_SCENARIO_CONTRACT.md)

## SOURCE: docs/spec/BENCHMARK_SCORECARD_GUIDE.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_SCORECARD_GUIDE.md](./appendices/benchmark/BENCHMARK_SCORECARD_GUIDE.md)

## SOURCE: docs/spec/BENCHMARK_TYPES.md
# Superseded by benchmark cluster contract

- Superseded by: [BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md](./BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md)
- Appendix source: [appendices/benchmark/BENCHMARK_TYPES.md](./appendices/benchmark/BENCHMARK_TYPES.md)

## SOURCE: docs/spec/CPU_MEMORY_BUDGET_MODEL.md
# CPU and Memory Budget Model

Runtime budget controls:

- `jobs`: upper bound on parallel dispatch width
- `cpu_budget`: aggregate CPU budget for batch scheduling
- node resource request: `cpu` and `mem_mb` contracts from graph node resources

Dispatch rule:

- a node is dispatch-eligible only if adding it does not exceed current CPU budget
- blocked nodes remain visible in scheduler blocked-by-budget diagnostics


## SOURCE: docs/spec/PERFORMANCE_CONTRACT.md
# Performance Contract

## Scope
Defines benchmark classes, canonical system scenarios, evidence requirements, and claim discipline.

## Related contracts
- `docs/spec/BENCHMARK_SCENARIO_CONTRACT.md`
- `docs/spec/BENCHMARK_REPRODUCIBILITY_CONTRACT.md`
- `docs/spec/COMPARISON_METHOD_CONTRACT.md`
- `docs/spec/EVIDENCE_PUBLICATION_CONTRACT.md`

## Allowed claims
- Performance language in documentation must reference:
  - `evidence/perf/scenarios/` canonical workloads, and
  - `artifacts/benchmarks/` or `evidence/perf/baselines/` evidence.
- Claims about speed, efficiency, or low overhead without evidence links are non-compliant.

## Benchmark classes
- microbenchmark: isolated crate-level measurement.
- system benchmark: end-to-end DAG command execution with run artifacts.

## Canonical system scenarios
- tiny: `evidence/perf/scenarios/tiny_canonical.json`
- medium: `evidence/perf/scenarios/medium_canonical.json`
- wide: `evidence/perf/scenarios/wide_canonical.json`
- deep: `evidence/perf/scenarios/deep_canonical.json`
- 10k nodes: `evidence/perf/scenarios/tenk_nodes_canonical.json`
- large artifact: `evidence/perf/scenarios/large_artifact_canonical.json`
- cache-heavy: `evidence/perf/scenarios/cache_heavy_canonical.json`
- failure injection: `evidence/perf/scenarios/failure_injection_canonical.json`
- replay: `evidence/perf/scenarios/replay_canonical.json`
- diff: `evidence/perf/scenarios/diff_canonical.json`
- portability: `evidence/perf/scenarios/portability_canonical.json`

## Score benchmarks
- determinism score: `evidence/perf/scenarios/determinism_score.json`
- replay fidelity score: `evidence/perf/scenarios/replay_fidelity_score.json`
- explainability quality: `evidence/perf/scenarios/explainability_quality.json`
- artifact lineage completeness: `evidence/perf/scenarios/artifact_lineage_completeness.json`
- portability success-rate: `evidence/perf/scenarios/portability_success_rate.json`

## Latency benchmarks
- inspect-history latency: `evidence/perf/scenarios/inspect_history_latency.json`

## Battle scenarios
- scheduler overhead on many tiny tasks: `evidence/perf/scenarios/many_small_nodes_scheduler_overhead.json`
- artifact write amplification: `evidence/perf/scenarios/manifest_trace_write_amplification.json`
- replay verification cost: `evidence/perf/scenarios/replay_verification_cost.json`

## Structured output
- Baseline and measured benchmark outputs must satisfy:
  - `configs/schema/benchmarks/benchmark_report.schema.json`
- Required fields include benchmark format, machine metadata, commit SHA, and scenario results.

## Regression thresholds
- Trusted threshold policy is defined in `evidence/perf/baselines/regression_thresholds.json`.
- `bijux-dev-dag benchmark-compare` is the baseline comparison command.

## Related tests
- `crates/bijux-dev-dag/tests/benchmark_scenario_contract.rs`
- `crates/bijux-dag-runtime/tests/performance_capacity_contracts.rs`

## Versioning and change policy
- Changes to scenario semantics require updating scenario owner/version metadata.
- Schema changes require compatible reader strategy and migration notes.

## SOURCE: docs/spec/PERFORMANCE_OPTIMIZATION_CONTRACT.md
# Performance Optimization Contract

## Purpose

Define required optimization evidence for execution performance, resource efficiency, and regression resilience across core bijux-dag workloads.

## Required optimization coverage

- graph parsing and DAG validation benchmark evidence
- planner and scheduler latency benchmark evidence
- runtime node execution overhead benchmark evidence
- artifact hash and artifact IO throughput benchmark evidence
- replay, diff, explain, and run history benchmark evidence
- provenance traversal and artifact store benchmark evidence
- memory and CPU profiling evidence
- regression detection and trend reporting evidence

## Required governance artifacts

- performance optimization regression corpus
- performance optimization regression suite definition
- performance telemetry report
- performance trend report
- performance optimization checklist
- performance regression summary report

## Required verification surfaces

- machine-readable corpus and suite parsing contracts
- benchmark completion contracts in `bijux-dev-dag`
- release-visible optimization reports under `docs/reports/foundation`

## SOURCE: docs/spec/PERFORMANCE_STRATEGY.md
# Performance strategy

Primary authority: `docs/spec/PERFORMANCE_CONTRACT.md`

## Allowed performance claims

Performance claims are only allowed when backed by committed benchmark evidence artifacts.

## Benchmark classes

- parse
- validate
- plan
- execute-local
- replay
- import
- export
- manifest-finalize
- cache-lookup

## Benchmark architecture

- microbenchmarks: crate-level, criterion-based, isolated operations
- system benchmarks: end-to-end command workflows with run artifacts

## Required benchmark evidence fields

- benchmark format version
- machine metadata
- rust toolchain version
- commit SHA
- benchmark scenario id
- benchmark class
- run configuration
- measured durations and throughput

## Governance rules

- docs may not claim performance quality without benchmark artifact links
- regression analysis must compare against baseline artifacts under `evidence/perf/baselines/`
- smoke timings are not benchmark evidence

## SOURCE: docs/spec/RESOURCE_PROFILE_STRATEGY.md
# Resource profile strategy

## Resource dimensions

Resource evidence tracks:

- wall time
- CPU time (where measurable)
- RSS and peak memory (where measurable)
- artifact bytes
- trace bytes
- process count

## Measurement quality levels

- authoritative: measured directly from runtime/process telemetry in controlled environment
- approximate: derived from filesystem size, wall-clock timing, or host-level summary sampling

## Scenario coverage

Resource profile scenarios must include parse/validate pressure, execution pressure, manifest/trace growth,
cache metadata growth, replay, and import/export memory behavior.

## Budget policy

- scenario-level artifact size budgets and trace/manifest budgets are defined under `evidence/perf/scenarios/`.
- budget checks run in warning mode first and can be promoted to gate mode.

## Evidence outputs

Each benchmark report should include resource profile sections where feasible and
must declare measurement quality as `authoritative` or `approximate`.

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_MINIMALISM_POLICY.md
# Benchmark Minimalism Policy

## Objective

Keep benchmark surfaces small, claim-focused, and cost-effective.

## Rules

1. Every benchmark must declare supported claim, gate class, and noise class.
2. Core-claim benchmarks must define threshold assertions.
3. Benchmark docs must reference active compact packs only.
4. New benchmarks must replace an existing benchmark or include explicit non-overlap justification.

## Enforcement

- `configs/policy/benchmark_signal_governance.json`
- `configs/suites/benchmark_minimalism_verification.json`
- `crates/bijux-dev-dag/tests/benchmark_minimalism_guarantees_contracts.rs`

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_RAW_DATA_RETENTION.md
# Benchmark Raw Data Retention

## Purpose
Define retention policy for benchmark raw outputs and derived summaries.

## Retention requirements
- Raw benchmark reports must remain available for every published benchmark claim.
- Derived reports and scorecards must reference the raw report locations.
- Raw data must be stored under committed evidence or reproducible artifact paths.

## Required link targets
- `evidence/perf/baselines/`
- `artifacts/benchmarks/` (when produced in CI)
- `evidence/reports/` for derived scorecards and comparisons

## Deletion policy
Raw reports may be compacted only when a successor baseline is committed and references are updated.

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_REPRODUCIBILITY_CONTRACT.md
# Benchmark Reproducibility Contract

## Purpose
Define minimum reproducibility requirements for benchmark evidence publication.

## Required metadata in benchmark outputs
Benchmark reports must contain:
- benchmark format version
- machine metadata
- rust toolchain version
- commit SHA
- scenario identifier
- command line or harness invocation
- measured values

Schema authority: `configs/schema/benchmarks/benchmark_report.schema.json`

## Reproducibility requirements
- A benchmark claim is valid only when the scenario ID exists in `evidence/perf/scenario_registry.json`.
- Reports used for regression comparison must reference an existing baseline in `evidence/perf/baselines/`.
- Benchmark results cannot be represented as release evidence unless scenario metadata marks them `release_blocking: true` in `evidence/perf/metadata.json`.

## Verification hooks
- `cargo run -p bijux-dev-dag -- performance-evidence-report`
- `cargo run -p bijux-dev-dag -- benchmark-compare --current <report> --baseline <report>`

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_RESULT_FORMAT.md
# Benchmark Result Format

## Purpose
Define the stable benchmark report shape, including raw measurements and metadata required for reproducibility.

## Schema authority
- `configs/schema/benchmarks/benchmark_report.schema.json`

## Required report sections
- report metadata: format version, timestamp, commit SHA, toolchain
- machine metadata: CPU, memory, host context
- scenario metadata: scenario id, benchmark class, run configuration
- raw outputs: measured values, units, sample counts
- derived outputs: summaries and ratio comparisons

## Source-of-truth rule
Published benchmark summaries must retain links to raw report files used to generate them.

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_SCENARIO_CONTRACT.md
# Benchmark Scenario Contract

## Purpose
Define the canonical benchmark scenario registry for bijux-dag so benchmark claims always map to committed, versioned workloads.

## Required scenario fields
Every scenario JSON under `evidence/perf/scenarios/` must include:
- `scenario_id`
- `class`
- `size`
- `version`
- `owner`
- `graph`

## Registry authority
- Scenario registry file: `evidence/perf/scenario_registry.json`
- Performance metadata authority: `evidence/perf/metadata.json`
- Performance claim authority: `docs/spec/PERFORMANCE_CONTRACT.md`

## Required benchmark scenario identifiers
The registry must include the following scenario IDs:
- `tiny-canonical`
- `wide-canonical`
- `deep-canonical`
- `tenk-nodes-canonical`
- `large-artifact-canonical`
- `cache-heavy-canonical`
- `failure-injection-canonical`
- `replay-canonical`
- `diff-canonical`
- `portability-canonical`
- `determinism-score`
- `replay-fidelity-score`
- `explainability-quality`
- `artifact-lineage-completeness`
- `portability-success-rate`
- `inspect-history-latency`

## Evolution policy
- Changes to scenario meaning require `version` bump in the scenario file.
- Scenario removal must keep an archived reference in `evidence/inventory/benchmark_scenarios.md`.

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_SCORECARD_GUIDE.md
# Benchmark Scorecard Guide

How to read benchmark scorecards:
- focus on trend direction across comparable scenarios
- treat single-run spikes as noise unless repeated
- use regression thresholds from `configs/policy/benchmark_regression_thresholds.json`
- only publish performance claims when raw benchmark data is linked

## SOURCE: docs/spec/appendices/benchmark/BENCHMARK_TYPES.md
# Benchmark Types

## Microbenchmarks
- crate-level, low-level, isolated operations
- useful for regression localization

## Scenario benchmarks
- user-facing command and workflow latency
- used for product-level performance claims and release decisions
