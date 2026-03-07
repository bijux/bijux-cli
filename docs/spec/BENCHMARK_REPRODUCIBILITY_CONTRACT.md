# Benchmark Reproducibility Contract

## Purpose
Define minimum reproducibility requirements for benchmark evidence publication.

## Required metadata in benchmark outputs
Benchmark reports must contain:
- benchmark format version
- machine metadata
- rust toolchain version
- commit SHA
- scenario identifier
- command line or harness invocation
- measured values

Schema authority: `configs/schema/benchmarks/benchmark_report.schema.json`

## Reproducibility requirements
- A benchmark claim is valid only when the scenario ID exists in `evidence/perf/scenario_registry.json`.
- Reports used for regression comparison must reference an existing baseline in `evidence/perf/baselines/`.
- Benchmark results cannot be represented as release evidence unless scenario metadata marks them `release_blocking: true` in `evidence/perf/metadata.json`.

## Verification hooks
- `cargo run -p bijux-dev-dag -- performance-evidence-report`
- `cargo run -p bijux-dev-dag -- benchmark-compare --current <report> --baseline <report>`
