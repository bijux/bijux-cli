# Development

## Purpose
This guide provides practical guidance for contributors working on bijux-cli. It exists to reduce accidental violations of core invariants and to help new contributors make changes safely.

## Scope
It covers where to add commands, how to test, and how to avoid breaking core guarantees. It does not cover project governance or release processes.

## Where to Add Commands
Add new CLI commands under `src/bijux_cli/cli/commands/` and keep command handlers thin. Core logic should live in services so it can be tested independently of CLI wiring.

## Where Not to Add Logic
Do not add policy decisions or output routing in infra modules. Those decisions must remain in core so tests can enforce invariants consistently.

## Testing Expectations
Run unit and regression tests for the areas you touched, and confirm that core invariants remain intact. If you change behavior, add or update tests that assert the new guarantees.

## Tooling Notes
The project enforces linting, quality checks, and security scans. These checks are not optional; they preserve the guarantees documented in the concepts section.

## Build And Release Discipline

Prefer deterministic, repository-local workflows:

- build from a clean checkout
- use pinned toolchains and lockfiles
- avoid relying on mutable host state
- record release artifacts and checksums from the tagged commit

Minimal local build verification:

```bash
cargo build --locked --workspace
cargo test --locked --workspace
```

## Release Review

Before tagging a release, confirm:

1. CI is green for the release commit.
2. Release evidence artifacts exist and are current.
3. Parity and known-gap artifacts have been reviewed.
4. Package-health and runtime-identity reports are green.
5. Docs build succeeds.
6. `bijux version` and `bijux cli doctor` pass in a clean environment.

Use generated artifacts rather than handwritten status summaries:

- `artifacts/status/release_evidence_bundle.json`
- `artifacts/status/release_status_manifest.json`
- `artifacts/status/release_truth_report.json`
- `artifacts/status/docs_audit.json`
- `artifacts/status/test_quality_audit.json`
- `artifacts/parity/command_parity_matrix.json`

## Rollback And Compatibility Checks

If a release candidate or published version regresses:

1. stop promoting the new version
2. reinstall the last known-good version through the same channel
3. verify with `bijux version`, `bijux cli paths`, and `bijux cli doctor`
4. pin the rollback version in CI and deployment manifests
5. capture the regression with impact and reproduction details

For maintainer-facing compatibility review, prefer generated evidence and `dev`
surfaces:

```bash
bijux dev cli runtime-identity --json --no-pretty
bijux dev cli parity --json --no-pretty
```

Review:

- `artifacts/status/runtime_unity_report.json`
- `artifacts/parity/binary_vs_python_bridge_parity_report.json`
- `artifacts/parity/command_parity_matrix.json`

## References
- [Contributor mental model](contributor-mental-model.md)
- [Architecture walk-through](../architecture/walkthrough.md)
