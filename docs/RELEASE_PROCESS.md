# Release process

## Validation gates

`release verify` executes:
- formatting and linting guard
- dependency policy checks
- dependency resolution checks
- workspace tests
- golden/runtime compatibility checks

## Public API verification

Public API baselines under `docs/api/*.txt` are validated during release verification.

## Artifact and report policy

- Run artifacts are written under `artifacts/`.
- CI and local release paths should include `--report` output for machine-readable results.
