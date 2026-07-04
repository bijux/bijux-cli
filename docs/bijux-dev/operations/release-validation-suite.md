---
title: Release Validation Suite
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-05
---

# Release Validation Suite

This page describes the canonical local verification lane for release
readiness in `bijux-core`.

The suite validates the committed release candidate tree instead of the live
worktree. That keeps unrelated local edits out of release evidence and makes
local runs match the publish surface that CI evaluates.

## Local Entrypoints

- `make release-validate-rs`
- `cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify`

`make release-validate-rs` owns the canonical Rust release validation suite.
`bijux-dev-cli release verify` runs that suite first, then refreshes the
release readiness report and compatibility matrix.

## Suite Commands

The suite runs these commands from a clean tree prepared from committed `HEAD`:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-targets --all-features --locked`
- `cargo doc --workspace --all-features --no-deps`
- `cargo package --list` for `bijux-dag-core`, `bijux-dag-artifacts`, `bijux-dag-runtime`, `bijux-dag-app`, and `bijux-dag-cli`
- `cargo publish --dry-run --locked` for `bijux-dag-core`, `bijux-dag-artifacts`, `bijux-dag-runtime`, `bijux-dag-app`, and `bijux-dag-cli`
- `cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture`

## Output Surface

- clean release tree: `artifacts/rust/release-validation/<run-id>/workspace/`
- shared target dir: `artifacts/rust/release-validation/<run-id>/target/`
- command logs: `artifacts/rust/release-validation/<run-id>/`
- release readiness report: `artifacts/release/readiness_report.json`
- compatibility matrix: `artifacts/release/compatibility_matrix.json`

## Failure Rule

- treat suite failures as release blockers until the committed candidate passes
- do not replace this suite with ad hoc local commands during release review
- keep public DAG crates publishable without pulling repository-internal test support crates into crates.io release validation

## Next Reads

- [Release Operations](release-operations.md)
- [CI and Automation](ci-and-automation.md)
- [Release Validation Workflow](../gh-workflows/release-validation.md)
- [Release Surfaces](../makes/release-surfaces.md)
