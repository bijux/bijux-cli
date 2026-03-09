# CI Integration

Define the contract for integrating bijux-dag execution and validation into CI pipelines.

CI is the operational control point for consistent DAG validation, deterministic execution checks, and release readiness.

## Explanation
CI pipeline baseline stages:
1. environment setup and toolchain pinning.
2. DAG/schema validation.
3. deterministic test and replay/diff checks.
4. quality gates and publishing decisions.

Recommended CI job topology:
- `validate`: static checks, DAG/schema validation, basic command surface verification.
- `test`: unit/integration lanes and fixture-backed behavior checks.
- `determinism`: selected replay/diff checks against known baselines.
- `release-readiness`: strict gate for tagged builds.

Recommended stage ordering:
1. `validate` (fast fail on shape/contracts)
2. `test` (unit/integration correctness)
3. `determinism` (replay/diff confidence)
4. `release-readiness` (promotion decision)

CI requirements:
- pin language/runtime versions to reduce drift.
- persist essential test/replay artifacts for post-failure diagnostics.
- publish concise run summary with failure reason classification.

Failure handling:
- fail fast for schema/contract violations.
- classify runtime failures separately from infrastructure failures.
- retain enough context for deterministic reproduction.

What CI is expected to prove:
- command and contract surfaces remain healthy for covered lanes.
- selected replay/diff checks remain classification-consistent.
- release gating policy is applied consistently.

What CI cannot prove by itself:
- universal backend equivalence outside tested capability envelope.
- absence of environment-specific failures outside covered matrix.
- correctness of external systems not represented in test evidence.

## Examples
```yaml
name: ci
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p bijux-dev-dag -- docs validate
      - run: cargo test --workspace --locked
  determinism:
    runs-on: ubuntu-latest
    needs: [validate]
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p bijux-dev-dag -- foundation --advisory
  release-readiness:
    if: startsWith(github.ref, 'refs/tags/')
    runs-on: ubuntu-latest
    needs: [validate, determinism]
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p bijux-dev-dag -- release artifact-verify
```

```text
Expected CI summary fields:
- commit_sha
- lane_statuses
- replay_or_diff_gate
- final_decision
```

## Guarantees
- CI integration contract defines a reproducible and auditable execution path.
- Failure classes remain distinguishable for faster remediation.
- Determinism checks can be enforced as explicit release gates.
- Includes concrete GitHub Actions topology usable as baseline.

## Limitations
- This document does not mandate one CI vendor.
- Lane composition may differ by repository size or release policy.
- Network and host volatility can still cause infrastructure-level flakiness.
- Example workflow is illustrative and may require repository-specific command adjustments.

## Related
- `docs/07-operations/02-reproducible-builds.md`
- `docs/07-operations/05-backend-support.md`
- `docs/08-development/02-testing-strategy.md`
- `docs/08-development/04-contributing.md`

## Integrating bijux-dag into existing CI systems

Adopt bijux-dag in incremental layers:

1. add DAG validation to existing lint/validate jobs,
2. add run/test evidence capture to existing test jobs,
3. add replay/diff gates only for release-critical workflows.

This avoids disruptive pipeline rewrites while still improving evidence quality.

Cross-CI adaptation pattern:

- map `validate`, `test`, `determinism`, `release-readiness` stages to native job primitives,
- preserve artifact retention for failing lanes,
- emit one normalized summary for gate decisions regardless of CI vendor.
