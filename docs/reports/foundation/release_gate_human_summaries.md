# Release Gate Human Summaries

- `fmt`: verifies code formatting contract and fails on style drift.
- `lint`: verifies lint/static checks and fails on unsafe or disallowed patterns.
- `audit`: verifies dependency/supply-chain policy compliance.
- `test`: verifies fast-lane regression safety.
- `test-all`: verifies full-lane regression safety including slow/ignored scope.
- `coverage`: verifies line-coverage policy and protected-file constraints.
- `evidence-all`: verifies evidence governance surfaces and release-evidence integrity.
