# Runtime Execution Conformance Suite

This suite focuses on runtime execution behavior separate from governance checks.

Coverage groups:

- stable scheduling under equal priority sets
- deterministic scheduling under varied concurrency budgets
- cache-hit and skipped-node scheduling consistency
- failure propagation semantics on fan-in/fan-out workflows
- timeout versus subprocess-exit race handling
- cancellation semantics under concurrency
- monotonic event timestamp checks
- bounded event volume for retry-heavy runs
- artifact commit atomicity under interruption

