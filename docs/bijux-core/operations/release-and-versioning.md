---
title: Release and Versioning
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-05
---

# Release and Versioning

Release work at the repository root coordinates tags, publication workflows,
docs deployment, and release evidence across all published surfaces.

```mermaid
sequenceDiagram
    participant Maintainer
    participant Repo as Repository
    participant CI as CI workflows
    participant Registry as Release targets
    participant Consumer

    Maintainer->>Repo: merge reviewed release-ready change
    Repo->>CI: trigger tag and publish workflows
    CI->>CI: run release validations
    CI->>Registry: publish versioned artifacts and release metadata
    Registry-->>Consumer: release becomes consumable
```

## Current Root Responsibilities

- tag-driven publication for crates.io, PyPI, and GitHub releases
- documentation deployment from `main`
- release-tree stamping and release-note generation
- coordination between CI health and publish decisions

The current DAG operator-facing release framing for the `v0.4.0` line lives in
[v0.4.0 Release Notes](../../bijux-dag/operations/v0-4-0-release-notes.md).

## Release Notes Minimum Template

Every release summary should stay evidence-backed and operator-readable.

```md
# Release <version>

## Summary

- explain what changed in one or two concrete bullets

## Public Behavior Changes

- name operator-visible command, API, schema, or artifact changes
- link the handbook, contract, or migration page for each change

## Compatibility Notes

- state whether the release is fully compatible, conditionally compatible, or intentionally breaking
- name the affected surface and required operator response

## Migration Steps

- list the exact commands, config changes, or rollout actions operators must perform
- say explicitly when no migration is required

## Known Limitations

- link limitations that still apply after the release
- explain whether the release changes the workaround or target audience

## Evidence

- list the release reports, validation artifacts, or benchmark evidence behind the notes
```

## Release Rule

Never document a release path without naming the workflow file or make target
that actually carries it.

## Next Reads

- [Automation Surfaces](automation-surfaces.md)
- [Artifact Governance](artifact-governance.md)
- [Core Release and Versioning](../governance/release-and-versioning.md)
