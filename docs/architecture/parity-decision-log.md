# Parity Decision Log

Parity decisions are artifact-backed.

Primary sources:
- `artifacts/parity/command_parity_matrix.json`
- `artifacts/parity/command_parity_diffs.json`
- `artifacts/parity/parity_regression_diffs.json`
- `docs/architecture/parity/intentional_differences.json`

Rule:
- no command may regress from `complete` to `partial` or `missing` without an explicit matrix and diff update in the same change.
