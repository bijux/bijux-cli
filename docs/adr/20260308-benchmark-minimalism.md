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
