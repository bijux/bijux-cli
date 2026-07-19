---
title: ci
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-07
---

# ci

`ci.yml` is the main push and pull-request verification entrypoint for the
repository.

## Trigger

- `push` on `main`
- every `pull_request`

## Job Shape

- provisions Rust `1.86.0` through the shared `RUST_TOOLCHAIN_VERSION` workflow environment
- `Formatting` runs `make gh-fmt`
- `Lint` runs `make gh-lint`
- `Security` installs pinned Rust security tools and runs `make gh-security`
- `Tests` runs the Python-version matrix through `make gh-test`
- `make gh-test` routes Rust through `make test-release-rs`, which uses the `ci` nextest profile and excludes governed experimental and internal DAG portfolios from the required release lane
- release-candidate package, publish, doc, and smoke validation run in `release-validation.yml` through `make gh-release-validate`

## Next Reads

- [Documentation Deployment](deploy-docs.md)
- [release-validation](release-validation.md)
- [release-crates](release-crates.md)
- [CI Targets](../makes/ci-targets.md)
