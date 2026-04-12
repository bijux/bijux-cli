---
title: CI Targets
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# CI Targets

GitHub Actions jobs should delegate shell behavior to make targets so local and
hosted verification stay aligned.

## CI-Aligned Targets

- `make gh-fmt`
- `make gh-lint`
- `make gh-security`
- `make gh-test`
- `make gh-docs-install`
- `make gh-release-wait-for-ci`

## CI Rule

When a workflow grows shell logic that make already owns, move that logic back
to the make layer and keep the workflow file thin.

## Next Reads

- [Release Surfaces](release-surfaces.md)
- [gh-workflows](../gh-workflows/index.md)
- [CI and Automation](../operations/ci-and-automation.md)
