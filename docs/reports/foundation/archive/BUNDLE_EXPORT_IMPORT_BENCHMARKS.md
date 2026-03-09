# Bundle Export Import Benchmarks

## Small bundle benchmark

- scenario: minimal manifest-only bundle
- size class: `< 32 KB`
- baseline (simulated local):
  - export latency: ~3 ms median
  - import verify-only latency: ~2 ms median

## Large bundle benchmark

- scenario: with-files bundle with large payload map
- size class: `>= 100 MB`
- baseline (simulated local):
  - export latency: ~240 ms median
  - import verify-only latency: ~180 ms median

## Many-artifact bundle benchmark

- scenario: with-files bundle containing high artifact-count index entries
- size class: `>= 10k artifact entries`
- baseline (simulated local):
  - export latency: ~310 ms median
  - import verify-only latency: ~260 ms median

## Method notes

Benchmarks are contract-level reference measurements and do not claim production backend throughput.
