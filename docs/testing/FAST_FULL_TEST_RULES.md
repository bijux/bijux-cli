# Fast and Full Test Lane Rules

generated_from: `Makefile + crates/*/tests`

## Lane Definition

- `make test` is for deterministic, local-only, high-signal checks.
- `make test-all` includes full Rust suites (including ignored tests).
- `make test-release` adds release-critical evidence and release gates.

## Fast Lane Criteria

- deterministic results
- no network dependency
- no external backend binary requirement
- no long-running benchmark dependency

## Full Lane Criteria

- release evidence generation
- comprehensive smoke coverage
- long-running conformance and benchmark gates
