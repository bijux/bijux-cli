# Release process

Authoritative release policy: `docs/spec/RELEASE_POLICY.md`.

## Validation gates

`release verify` executes:
- formatting and linting guard
- dependency policy checks
- dependency resolution checks
- workspace tests
- golden/runtime compatibility checks
- release readiness report generation
- compatibility matrix generation

## Public API verification

Public API baselines under `docs/api/*.txt` are validated during release verification.

## Control-plane release workflows

- `cargo run -p bijux-dev-dag -- release readiness`
- `cargo run -p bijux-dev-dag -- release compatibility-matrix`
- `cargo run -p bijux-dev-dag -- release evidence-bundle`
- `cargo run -p bijux-dev-dag -- release reproducibility-check --tag <tag>`
- `cargo run -p bijux-dev-dag -- release post-release-verify [--binary <path>]`

## Artifact and report policy

- Run artifacts are written under `artifacts/`.
- CI and local release paths should include `--report` output for machine-readable results.
- Release notes use template `docs/reference/RELEASE_NOTE_TEMPLATE.md`.
- Known limitations must be updated in `docs/tracking/KNOWN_LIMITATIONS.md`.
- CLI JSON compatibility report must be updated in `docs/reports/foundation/archive/CLI_JSON_COMPATIBILITY_REPORT.md`.
- Schema compatibility review must be completed using:
  - `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`
  - `docs/reports/foundation/archive/SCHEMA_CHANGELOG.md`
  - compatibility fixtures under `evidence/compat/*_schema/`
