---
title: Package Ownership
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Package Ownership

Package ownership keeps responsibilities stable and prevents hidden coupling
between product and maintainer layers.

## Visual Summary

```mermaid
flowchart LR
    cli_pkg[bijux-cli] --> cli_docs[cli handbook ownership]
    dag_pkg[bijux-dag-*] --> dag_docs[dag handbook ownership]
    dev_pkg[bijux-dev] --> dev_docs[maintainer handbook ownership]
    core_docs[repository handbook] --> all[shared policy and boundaries]
```

## Ownership Map

- `crates/bijux-cli`: CLI command runtime and plugin behavior
- `crates/bijux-dag-*`: DAG parse/run/replay/diff and artifact behavior
- `crates/bijux-cli-python`: Python package and bridge behavior
- `crates/bijux-dev`: maintainer automation, suites, and evidence reports

## Ownership Rules

- ownership claims must map to real crate/module paths
- cross-crate changes require coordination in all owning docs
- packages must not silently redefine another package's public contract

## Code Anchors

- `crates/bijux-cli/CONTRACT.md`
- `crates/bijux-dag-app/CONTRACT.md`
- `crates/bijux-dev/CONTRACT.md`
- `docs/bijux-cli/index.md`

## Next Reads

- [Change Management](change-management.md)
- [Decision Record Policy](decision-record-policy.md)
- [Dependency Direction](../architecture/dependency-direction.md)
