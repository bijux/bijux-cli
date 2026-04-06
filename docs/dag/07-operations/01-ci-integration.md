# CI Integration

This guide defines how to integrate bijux-dag into CI so pipeline results are promotable evidence, not just pass/fail noise.

## What CI should prove

A correct CI integration should prove, for the code and matrix you executed:
- DAG definitions are valid and schedulable.
- runtime behavior matches expected test outcomes.
- replay/diff classification is stable for required release scopes.
- release gates were evaluated with explicit evidence.

CI does not prove:
- equivalence on untested backends,
- correctness of external systems not represented in evidence,
- security of unverified imported inputs.

## Baseline job model

Use four jobs, with strict dependency order:
1. `validate`: DAG/schema/command-surface checks.
2. `test`: unit and integration checks.
3. `determinism`: replay/diff/proof checks against approved baselines.
4. `promote`: release decision from evidence summary.

A failed earlier job MUST block downstream promotion.

## Evidence contract per job

Each job should emit machine-readable records:
- `validate`: invalid-DAG counts, failing identifiers.
- `test`: failing suites, environment fingerprint.
- `determinism`: replay/diff classifications with reason codes.
- `promote`: final decision plus evidence references.

If `determinism` is skipped, `promote` must report that scope as not proven.

## Practical CI integrations

GitHub Actions example:

```yaml
name: bijux-ci
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p bijux-core-dev -- dag validate examples/pipeline.dag.json
  test:
    runs-on: ubuntu-latest
    needs: [validate]
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --workspace --locked
  determinism:
    runs-on: ubuntu-latest
    needs: [test]
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p bijux-core-dev -- run replay --baseline runs/r_120 --mode strict
      - run: cargo run -p bijux-core-dev -- run diff --baseline runs/r_120 --candidate runs/latest --json
  promote:
    if: github.ref_type == 'tag'
    runs-on: ubuntu-latest
    needs: [determinism]
    steps:
      - uses: actions/checkout@v4
      - run: cargo run -p bijux-core-dev -- release verify --require-equivalent
```

Jenkins/GitLab/CircleCI mapping pattern:
- keep the same four logical jobs,
- preserve artifact retention on failure,
- produce one normalized evidence summary independent of vendor.

## CI anti-patterns

Avoid these patterns:
- running replay/diff only on green tags with no pull-request coverage,
- passing promotion when replay/diff output is `unknown` or missing,
- mixing infrastructure failure with semantic drift in one generic failure status,
- deleting failed-run evidence to save storage.

## Guarantees

- This integration model yields auditable promotion decisions.
- Replay/diff evidence is part of the release gate, not a side report.
- Missing determinism evidence is explicit.

## Non-guarantees

- CI success is not universal backend certification.
- CI success is not a substitute for runtime trust-boundary checks.

## Next reading

- [Reproducible builds](02-reproducible-builds.md)
- [Security model](03-security-model.md)
- [Trust boundaries](04-trust-boundaries.md)
