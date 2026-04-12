---
title: deploy-docs
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# deploy-docs

`deploy-docs.yml` publishes the handbook site from `main` after rebuilding
docs, generated artifact pages, and navigation checks.

## Trigger

- `push` on `main`
- manual `workflow_dispatch`

## Job Shape

- install the docs toolchain with `make gh-docs-install`
- generate release artifact summary pages
- run `make docs-check`
- configure the Git author
- deploy via `make docs-deploy`

## Next Reads

- [ci](ci.md)
- [release-github](release-github.md)
- [Docs Operations](../operations/docs-operations.md)
