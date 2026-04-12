---
title: ci
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# ci

`ci.yml` is the main push and pull-request verification entrypoint for the
repository.

## Trigger

- `push` on `main`
- every `pull_request`

## Job Shape

- `Formatting` runs `make gh-fmt`
- `Lint` runs `make gh-lint`
- `Security` installs pinned Rust security tools and runs `make gh-security`
- `Tests` runs the Python-version matrix through `make gh-test`

## Next Reads

- [deploy-docs](deploy-docs.md)
- [release-crates](release-crates.md)
- [CI Targets](../makes/ci-targets.md)
