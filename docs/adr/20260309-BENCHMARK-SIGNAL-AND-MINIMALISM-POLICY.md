# Benchmark Signal and Minimalism Policy

Status: accepted
Owner: performance maintainers
Date: 2026-03-09

## Decision
Benchmarking is claim-oriented and minimal. Scenarios and thresholds must map to explicit contract claims.

## Consequences
- Low-signal benchmark sprawl is disallowed.
- Release-critical performance claims require stable evidence.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-BENCHMARK-MINIMALISM.md
# ADR: Benchmark Minimalism

## Status

Accepted

## Context

Benchmark surfaces have grown across many scenario families, risking overlap and reduced signal efficiency.

## Decision

1. Organize benchmark coverage into five compact claim-oriented packs.
2. Keep core-claim benchmarks threshold-gated.
3. Require gate and noise classification for benchmark scenarios.
4. Require new benchmarks to replace an existing scenario or justify non-overlap.

## Consequences

- Benchmark governance becomes easier to review.
- Slow or noisy low-value scenarios are easier to retire.
- Core claim coverage remains stable and auditable.

## Enforcement

- `docs/spec/BENCHMARK_MINIMALISM_POLICY.md`
- `configs/suites/benchmark_minimalism_verification.json`
- `crates/bijux-dev-dag/tests/benchmark_minimalism_guarantees_contracts.rs`

### SOURCE: 20260308-BENCHMARK-SIGNAL-GOVERNANCE.md
# ADR: Benchmark Signal Governance

- Date: 2026-03-08
- Status: Accepted

## Context

Benchmark coverage existed across identity, replay, artifact, and runtime surfaces, but governance expectations were scattered. We needed one policy that ties each benchmark to a product claim, gate class, and stability expectation.

## Decision

Adopt benchmark signal governance policy at `configs/policy/benchmark_signal_governance.json` with required fields per scenario:

- `supported_claim`
- `gate_class` (`gating` or `advisory`)
- `noise_class` (`low`, `medium`, `high`)
- `threshold_assertion`
- `source_report`

Generated reports and contracts now enforce:

- claim-to-scenario coverage
- no orphan benchmark scenarios without release claim mapping
- no claim families without benchmark scenarios
- explicit noisy/flaky and slow/low-signal review outputs
- promotion/demotion candidate reports
- roadmap-pillar gap visibility
- benchmark docs must cite generated outputs only

## Consequences

- Release decisions can prioritize benchmark signals with explicit noise and gate classes.
- Advisory benchmark surfaces remain isolated from release blockers until promoted.
- Benchmark governance drift becomes test-detectable instead of review-time guesswork.

## Artifacts

- `configs/policy/benchmark_signal_governance.json`
- `docs/reports/foundation/benchmark_scenarios_by_claim_report.md`
- `docs/reports/foundation/benchmark_scenarios_without_release_claim_report.md`
- `docs/reports/foundation/release_claims_without_benchmark_scenario_report.md`
- `docs/reports/foundation/flaky_noisy_benchmark_report.md`
- `docs/reports/foundation/slow_benchmark_signal_value_report.md`
- `docs/reports/foundation/benchmark_advisory_to_gating_candidates_report.md`
- `docs/reports/foundation/benchmark_gating_to_advisory_candidates_report.md`
- `docs/reports/foundation/benchmark_threshold_assertions_runtime_helpers.json`
- `docs/reports/foundation/benchmark_trend_by_claim_family_report.md`
- `docs/reports/foundation/benchmark_gaps_by_roadmap_pillar_report.md`
