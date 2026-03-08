# Release Gate Overlap Report

Potential overlap areas:

- `test` and `test-all` both enforce test correctness; `test` is fast-lane subset while `test-all` is complete lane.
- `coverage` and `test-all` both run broad suites, but `coverage` uniquely enforces line coverage policy.
- `evidence-all` and `test-release` share release-evidence surfaces, but `test-release` is release path while `evidence-all` is governance aggregate.

No overlap currently requires gate removal.
