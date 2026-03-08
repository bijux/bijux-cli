# Release Gate Redundancy Decisions Report

Redundancy review outcome:

- Keep both `test` and `test-all` for developer velocity and full verification separation.
- Keep `coverage` as a stronger, distinct invariant (coverage policy) rather than merging into `test-all`.
- Keep `evidence-all` as release-supporting aggregate gate distinct from release-blocking `test-release` path.

No gate collapse performed in this revision.
