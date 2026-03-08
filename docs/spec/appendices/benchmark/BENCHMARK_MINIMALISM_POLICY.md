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
