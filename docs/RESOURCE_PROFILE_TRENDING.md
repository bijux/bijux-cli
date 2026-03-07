# Resource profile trending

Use benchmark report artifacts to build historical resource trend evidence.

## Commands

- summary:
  - `cargo run -p bijux-dev-dag -- resource-profile-summary --report artifacts/benchmarks/baseline.json`
- append trend entry:
  - `cargo run -p bijux-dev-dag -- resource-trend-append --report artifacts/benchmarks/baseline.json --trend evidence/perf/baselines/resource_trend_v1.json`

## Trend format

Trend series uses `resource-trend/v1` and stores per-commit scenario resource profiles.
