# Performance Budget Law

Performance claims are valid only for critical user-facing paths and must stay evidence-backed.

Frozen requirements:

1. Startup latency budgets are enforced for `version`, `status`, `doctor`, `plugins list`, `cli config get`, and `dev cli status`.
2. Startup latency budgets are enforced under degraded conditions: broken plugin registry, large plugin registry, large config, and large history.
3. Memory-use benchmarks remain enforced for key command payloads and REPL startup memory estimate.
4. Large JSON and YAML rendering paths stay under explicit rendering budgets.
5. CI gates only enforce critical-path thresholds, and reject vanity microbenchmark drift as a release criterion.

Evidence sources:

- `artifacts/status/performance_report.json`
- `artifacts/status/performance_regression_budget.json`
- `artifacts/status/performance_benchmark_policy.json`
- `crates/bijux-cli/tests/cli_surface/resilience/performance_realism_hardening.rs`
- `crates/bijux-cli-output/tests/output_rendering_performance.rs`
- `crates/bijux-cli/tests/cli_surface/repl/repl_startup_performance_budget.rs`
