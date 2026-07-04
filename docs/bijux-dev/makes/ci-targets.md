---
title: CI Targets
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-05
---

# CI Targets

GitHub Actions jobs should delegate shell behavior to make targets so local and
hosted verification stay aligned.

## CI-Aligned Targets

- `make gh-fmt`
- `make gh-lint`
- `make gh-security`
- `make gh-test`
- `make gh-release-validate`
- `make gh-docs-install`
- `make gh-release-wait-for-ci`

## Test Lane Mapping

- `make gh-test` runs the required Rust release lane through `make test-release-rs`
- `make gh-release-validate` runs the committed-`HEAD` release-candidate suite through `make release-validate-rs`
- `make test-release-rs` uses the `ci` nextest profile and is the required release-candidate Rust lane
- `make test-all-rs` is the full Rust verification lane and includes governed ignored DAG tests

## CI Rule

When a workflow grows shell logic that make already owns, move that logic back
to the make layer and keep the workflow file thin.

## Next Reads

- [Release Surfaces](release-surfaces.md)
- [gh-workflows](../gh-workflows/index.md)
- [CI and Automation](../operations/ci-and-automation.md)
