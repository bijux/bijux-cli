# Test Philosophy

This repo separates tests by intent, not just speed.

- Unit: fast, isolated, module-level behavior; no IO boundaries.
- Regression: functional + integration behavior below the CLI boundary.
- E2E: real CLI boundary, stateful sequences, invariant assertions.
- Nightly: long-running, fuzz, stress, or heavy property tests (located in `tests/night/`).
- Benchmark: performance baselines with explicit regression thresholds.

If a test can pass without spawning the real CLI, it is not E2E.
