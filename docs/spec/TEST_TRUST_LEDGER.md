# Test trust ledger

## Scope

This ledger defines trust-value classification and mandatory semantic surfaces for runtime tests.

## Trust-value classes

- `critical`: must-never-break trust proofs tied to correctness and safety boundaries.
- `useful`: meaningful contract coverage that supports regression detection.
- `shallow`: low-depth checks that remain only when they guard discoverability or catalog integrity.
- `cosmetic`: non-semantic checks; forbidden as progress metrics.
- `duplicate`: overlapping checks superseded by stronger trust surfaces.

## Normative policy

The normative policy file is `configs/policy/test_trust_ledger.json`.

## Enforcement rules

- Every runtime test file must be classified by exactly one trust-value class.
- `must_never_break` entries must exist and cannot be classified as `cosmetic` or `duplicate`.
- Required semantic surfaces must exist and remain mapped.
- Forbidden snapshot macros are rejected outside explicit allowlisted files.
- Foundation and repo governance checks must include the test-trust cleanup guard.
