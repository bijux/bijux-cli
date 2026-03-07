# Performance strategy

Primary authority: `docs/spec/PERFORMANCE_CONTRACT.md`

## Allowed performance claims

Performance claims are only allowed when backed by committed benchmark evidence artifacts.

## Benchmark classes

- parse
- validate
- plan
- execute-local
- replay
- import
- export
- manifest-finalize
- cache-lookup

## Benchmark architecture

- microbenchmarks: crate-level, criterion-based, isolated operations
- system benchmarks: end-to-end command workflows with run artifacts

## Required benchmark evidence fields

- benchmark format version
- machine metadata
- rust toolchain version
- commit SHA
- benchmark scenario id
- benchmark class
- run configuration
- measured durations and throughput

## Governance rules

- docs may not claim performance quality without benchmark artifact links
- regression analysis must compare against baseline artifacts under `benchmarks/baselines/`
- smoke timings are not benchmark evidence
