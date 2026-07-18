---
title: CI Targets
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-07
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
- `make test-rs` runs only non-ignored fast tests outside the `slow__` namespace and governed slow roster
- `make test-slow-rs` runs the complementary named and rostered slow lane
- `make test-all-rs` is the full Rust verification lane and includes governed ignored experimental and internal DAG portfolios

Use [Release Validation Suite](../operations/release-validation-suite.md) for
the exact release-candidate command inventory and artifact outputs behind
`make gh-release-validate`.

## Frozen Commit Gates

- `PINNED_REF=<ref> make test-all-frozen` starts the full Rust verification lane from a detached checkout of `<ref>`
- `PINNED_REF=<ref> make lint-frozen` starts the Rust lint gate from a detached checkout of `<ref>`
- `PINNED_REF=<ref> make audit-frozen` starts the dependency audit gate from a detached checkout of `<ref>`
- each frozen gate writes run state under `artifacts/<sha>/`, including the pinned source tree at `artifacts/<sha>/frozen-repo/`
- background process metadata lives under `artifacts/<sha>/background/`, including `<gate>.console.log`, `<gate>.pid`, and `<gate>.exit.status`
- frozen Rust runs isolate Cargo state and reports under `artifacts/<sha>/rust/`

## CI Rule

When a workflow grows shell logic that make already owns, move that logic back
to the make layer and keep the workflow file thin.

## Next Reads

- [Release Validation Suite](../operations/release-validation-suite.md)
- [Release Surfaces](release-surfaces.md)
- [gh-workflows](../gh-workflows/index.md)
- [CI and Automation](../operations/ci-and-automation.md)
