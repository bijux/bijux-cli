# Benchmark evidence and claim contract

**What this spec is not**: runtime implementation plan, benchmark tooling internals, or release process.

## Scope

This contract defines:

- benchmark claim classes and publication criteria
- scenario registry and reproducibility requirements
- result format requirements and retention expectations
- minimal evidence model for performance claims

## Canonical requirements

- All published performance claims must point to raw benchmark data and scenario metadata.
- Benchmarks are classified by claim class and threshold policy before publication.
- Raw benchmark artifacts are retained for the evidence link chain and may only be compacted with replacement baselines.
- Scenario contracts are the single source for scenario identity and meaning.

## Evidence and implementation links

- Evidence schema: `configs/schema/benchmarks/benchmark_report.schema.json`
- Benchmark suites and comparisons in `crates/bijux-dev-dag`.
- Evidence policy: `configs/policy/benchmark_signal_gov...` and related governance artifacts.

## Canonical appendices

- [reproducibility](./appendices/benchmark/BENCHMARK_REPRODUCIBILITY_CONTRACT.md)
- [result format](./appendices/benchmark/BENCHMARK_RESULT_FORMAT.md)
- [scenario contract](./appendices/benchmark/BENCHMARK_SCENARIO_CONTRACT.md)
- [types](./appendices/benchmark/BENCHMARK_TYPES.md)
- [scorecard guide](./appendices/benchmark/BENCHMARK_SCORECARD_GUIDE.md)
- [minimalism policy](./appendices/benchmark/BENCHMARK_MINIMALISM_POLICY.md)
- [raw data retention](./appendices/benchmark/BENCHMARK_RAW_DATA_RETENTION.md)
