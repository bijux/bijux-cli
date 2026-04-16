---
title: API and Schema Governance
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# API and Schema Governance

Repository-level schema work exists to keep package contracts, published docs,
and release expectations aligned.

```mermaid
flowchart TD
    behavior[package public behavior] --> schemas[shared contracts and schemas]
    schemas --> pinned[pinned schema and contract snapshots]
    pinned --> checks[drift and compatibility checks]
    checks --> review[review and merge decision]
    review --> behavior

    code_drift[behavior changed without schema update] --> checks
    schema_drift[schema changed without stated intent] --> checks
```

## Root Governance Surfaces

- shared contract assets under `contracts/`
- package contract tests in CLI, DAG, and maintainer crates
- documentation pages that explain compatibility commitments

## Governance Rule

If a schema or contract change crosses package boundaries, the repository
handbook should explain that change before release notes try to summarize it.

## Next Reads

- [Artifact Governance](artifact-governance.md)
- [Change Management](change-management.md)
- [Compatibility and Schema](../governance/compatibility-and-schema.md)
