# Release Gate Contributor Quick-Start

1. Run `make test` before pushing.
2. If touching policy/report generators, run `make test-all` and `make evidence-all`.
3. For gate failures, inspect generated reports under `docs/reports/foundation`.
4. Keep gate docs and workflow behavior aligned with `configs/policy/release_gate_governance.json`.
