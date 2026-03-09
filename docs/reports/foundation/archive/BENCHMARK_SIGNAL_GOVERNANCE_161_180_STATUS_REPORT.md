# Benchmark Signal Governance Status Report (161-180)

Generated: 2026-03-08

This report maps tasks 161-180 to shipped benchmark signal governance policy,
generated outputs, enforcement contracts, and ADR coverage.

## 161-164 benchmark inventory and gap reports

- `docs/reports/foundation/benchmark_scenarios_by_claim_report.md`
- `docs/reports/foundation/benchmark_scenarios_without_release_claim_report.md`
- `docs/reports/foundation/release_claims_without_benchmark_scenario_report.md`
- `docs/reports/foundation/benchmarks_without_regression_thresholds_report.md`

## 165-167 benchmark declaration governance rules

- `configs/policy/benchmark_signal_governance.json`
- Enforced fields:
  - `supported_claim`
  - `gate_class`
  - `noise_class`

## 168-171 benchmark quality and lane-change reports

- `docs/reports/foundation/flaky_noisy_benchmark_report.md`
- `docs/reports/foundation/slow_benchmark_signal_value_report.md`
- `docs/reports/foundation/benchmark_advisory_to_gating_candidates_report.md`
- `docs/reports/foundation/benchmark_gating_to_advisory_candidates_report.md`

## 172-175 threshold assertions by product claim family

- `docs/reports/foundation/benchmark_threshold_assertions_graph_identity.json`
- `docs/reports/foundation/benchmark_threshold_assertions_run_history.json`
- `docs/reports/foundation/benchmark_threshold_assertions_artifact_trace.json`
- `docs/reports/foundation/benchmark_threshold_assertions_runtime_helpers.json`

## 176-177 trend and roadmap-gap reports

- `docs/reports/foundation/benchmark_trend_by_claim_family_report.md`
- `docs/reports/foundation/benchmark_gaps_by_roadmap_pillar_report.md`

## 178 benchmark review checklist

- `docs/reference/BENCHMARK_REVIEW_CHECKLIST.md`

## 179 benchmark docs generated-output gate

- `docs/reports/foundation/benchmark_docs_generated_sources_guard.md`

## 180 ADR

- `docs/adr/20260308-BENCHMARK-SIGNAL-GOVERNANCE.md`
