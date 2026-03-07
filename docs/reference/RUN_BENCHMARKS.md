# Run Benchmarks

## Local workflow
1. `cargo run -p bijux-dev-dag -- performance-evidence-report`
2. `cargo run -p bijux-dev-dag -- benchmark-baseline`
3. `cargo run -p bijux-dev-dag -- benchmark-compare --current <current.json> --baseline <baseline.json>`

## CI workflow
- CI must run benchmark contract tests and performance evidence policy checks.
- Benchmark claims in docs must reference committed raw evidence.

## Required references
- `docs/spec/PERFORMANCE_CONTRACT.md`
- `docs/spec/BENCHMARK_RESULT_FORMAT.md`
- `docs/spec/BENCHMARK_RAW_DATA_RETENTION.md`
