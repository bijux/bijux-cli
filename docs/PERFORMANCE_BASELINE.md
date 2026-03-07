# Performance baseline evidence

Use `cargo run -p bijux-dev-dag -- benchmark-baseline` to record structured system benchmark results.
Use `cargo run -p bijux-dev-dag -- benchmark-compare --current <file> --baseline <file>` to report threshold regressions.

Baseline artifacts are stored under `artifacts/benchmarks/` and compared against
`evidence/perf/baselines/` references.

Performance claims must reference benchmark evidence artifacts. Timing smoke wrappers
without scenario metadata are not accepted as evidence.
