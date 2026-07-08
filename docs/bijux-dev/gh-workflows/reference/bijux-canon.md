---
title: bijux-canon
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-06
---

# bijux-canon

`bijux-canon.yml` is the repository governance workflow for DAG- and
evidence-heavy changes. Despite the name, it is part of `bijux-core` today and
acts as a wide governance matrix for DAG, evidence, coverage, and schema work.

## Trigger

- manual `workflow_dispatch`
- `push` and `pull_request` when DAG, maintainer, docs, evidence, or workflow
  files change

## Job Shape

- provision Rust `1.86.0` through the workflow `RUST_TOOLCHAIN_VERSION` pin
- run formatting, lint, governance tests, and compatibility drift checks
- verify evidence and generated reports
- run docs and public-surface drift checks
- exercise package dry runs and repository-health reports

## Naming Rule

This workflow should stay documented explicitly while the filename still
exists. If the file is renamed later, update this page in the same change.

## Next Reads

- [ci](../ci.md)
- [deploy-docs](../deploy-docs.md)
- [Maintainer Handbook](../index.md)
