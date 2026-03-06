# Evidence model

This file defines what counts as proof.

## Correctness evidence

- Deterministic contract tests
- Negative fixtures for rejected behavior
- Invariant checks and diagnostics snapshots

## Compatibility evidence

- Versioned schema fixtures
- Compatibility matrix fixtures
- Migration and downgrade checks

## Performance evidence

- Measured benchmark artifacts produced by benchmark suites
- Trend reports with workload metadata

`benchmark-baseline` is currently provisional and must not be used as a release guarantee by itself.

## Memory evidence

- Measured memory artifacts with explicit environment metadata
- Budget regression checks

`memory-smoke` is currently provisional and must not be used as a release guarantee by itself.

## Release readiness evidence

- Contract suite report
- Compatibility report
- Security and policy checks
- Rollback and migration verification artifacts

## Guarantee language rule

Any guarantee statement in repository docs must include a markdown link to proof in one of:
- `docs/spec/`
- test fixture/test file
- benchmark artifact/report
