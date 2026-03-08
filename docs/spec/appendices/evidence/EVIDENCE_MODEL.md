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

`benchmark-baseline` must carry workload metadata and environment context before it is used
as release proof.

## Memory evidence

- Measured memory artifacts with explicit environment metadata
- Budget regression checks

Memory evidence must come from measured benchmark/observability artifacts; standalone smoke
timing is not accepted as release proof.

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
