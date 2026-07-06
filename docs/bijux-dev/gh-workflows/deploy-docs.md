---
title: deploy-docs
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-06
---

# deploy-docs

`deploy-docs.yml` publishes the handbook site from `main` after rebuilding
docs, generated artifact pages, and navigation checks.

## Trigger

- `push` on `main`
- manual `workflow_dispatch`

## Job Shape

- install the docs toolchain with `make gh-docs-install`
- honor `.github/docs-deploy.env`, including `BIJUX_DOCS_RUST_TOOLCHAIN=1.86.0`
- generate release artifact summary pages
- run `make docs-check`
- configure the Git author
- deploy via `make docs-deploy`

## Next Reads

- [ci](ci.md)
- [release-github](release-github.md)
- [Docs Operations](../operations/docs-operations.md)
