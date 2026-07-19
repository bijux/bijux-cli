---
title: Release Operations
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Release Operations

Use this page when the repository is close to a release boundary and the next
question is sequence, ownership, and proof rather than implementation.

The release path is intentionally conservative. Every step exists so the tagged
result still matches the behavior, compatibility notes, and docs the
repository is prepared to stand behind in public.

Visible maintainer command ownership remains governed by
`contracts/foundation/maintainer_command_surface.v1.json`.

## Release Workflow Rules

- only tag commits with green required gates
- keep release lanes on Rust `1.86.0`, matching `Cargo.toml`, `rust-toolchain.toml`, and `ci.yml`
- include compatibility notes for CLI and DAG changes
- ensure docs navigation and links are valid before publishing
- verify post-release health and rollback readiness

## Current Publication Policy

Canonical package status and publish order are defined by
`contracts/foundation/workspace_package_boundary.v1.json` and
[Package Boundary](../../bijux-core/foundation/package-boundary.md).

- `v0.4.0` publishes `bijux-cli` to crates.io and PyPI.
- `v0.4.0` publishes the DAG Rust crates to crates.io in dependency order: `bijux-dag-core`, `bijux-dag-artifacts`, `bijux-dag-runtime`, `bijux-dag-app`, and `bijux-dag-cli`.
- GitHub Releases and GHCR publish two stamped release families: the `bijux-cli` distribution bundle and the `bijux-dag` binary tarball.
- `bijux-dag-testkit`, `bijux-dev`, and `bijux-cli-python` remain repository-internal support crates and are not published to crates.io.
- The canonical repository for both products is `https://github.com/bijux/bijux-core`.

## Sequence That Must Hold

| Step | What it should prove |
| --- | --- |
| release validation | the candidate commit is publishable from a clean release tree |
| compatibility and docs review | public readers can understand what changed and whether compatibility moved |
| tag and publish | published artifacts point back to the reviewed commit identity |
| post-release monitoring | the public result still behaves like the reviewed release lane predicted |

## Preflight Checklist

- required release-lane tests and maintainer verification commands are green
- `make release-validate-rs` is green before any release recommendation
- `make test-release-rs` is green before any release recommendation
- `make test-all-rs` is green whenever DAG experimental or internal ignored coverage changed or ignored-test governance changed
- `bijux-dev-cli maintenance ignored-dag-tests` reports `integrity_status: ok`
  whenever ignored-test governance, quarantined portfolios, or source-level DAG
  test helpers changed
- compatibility notes are prepared for changed public behavior
- documentation tree and MkDocs navigation are synchronized
- release owner and rollback owner are explicitly assigned

## Postflight Checklist

- published artifacts match tagged commit identity
- docs site builds and serves expected handbook routes
- no new unresolved failures in release-monitoring workflows

## Standard Commands

```bash
make release-validate-rs
make test-release-rs
make test-all-rs
cargo run -q -p bijux-dev --bin bijux-dev-cli -- maintenance ignored-dag-tests
cargo run -q -p bijux-dev --bin bijux-dev-cli -- docs write-dag-cli-reference
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify
make docs-check
```

## Release Validation Suite

Use [Release Validation Suite](release-validation-suite.md) for the canonical
release-candidate gate. That page owns the exact command inventory, execution
model, artifact outputs, and failure ownership for `make release-validate-rs`,
`make gh-release-validate`, and `cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify`.

At the release-operations level, the important rule is sequence: release
validation happens before tag creation and before any publish command is trusted
as release evidence.

## Reader Shortcut

If a tag, artifact, or release note gets ahead of release validation and
compatibility review, the repository has already broken sequence even if the
publish technically succeeds.

## Code Anchors

- `crates/bijux-dev/src/commands/cli_release_command.rs`
- `crates/bijux-dev/src/suites/release.rs`
- `.github/workflows/`

## Continue Reading

- [Release Validation Suite](release-validation-suite.md)
- [Core Release and Versioning](../../bijux-core/operations/release-and-versioning.md)
- [Contract Governance](../governance/contract-governance.md)
- [Known Limitations](../governance/known-limitations.md)
