# Truth Before Polish Milestone

## Freeze
This milestone freezes truth-reporting gates as non-optional release criteria.

## Required Artifacts
- `artifacts/status/what_is_done.json`
- `artifacts/status/what_is_left.json`
- `artifacts/status/what_is_partial.json`
- `artifacts/status/what_is_deferred.json`
- `artifacts/status/what_is_intentionally_different.json`
- `artifacts/parity/command_parity_matrix.json`
- `artifacts/status/docs_audit.json`
- `artifacts/status/test_quality_audit.json`

## Required Gate
- `scripts/status/enforce_release_truth_gates.py --enforce`

## Intent
No “complete” claim is valid unless these artifacts and gates support it.
