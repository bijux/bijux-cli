---
title: CI and Automation
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# CI and Automation

This page explains how local workflows and hosted pipelines stay aligned in
`bijux-core`.

The repository depends on that alignment because a green PR only means something
when the same work can be reproduced before the push as well.

## Automation Flow

```mermaid
flowchart TD
    local["local gate execution"] --> pr["PR workflows"]
    pr --> required["required status checks"]
    required --> release["release workflow triggers"]
```

## Automation Scope

- build, test, and contract workflows
- docs build and publish workflows
- release workflows for crate and package publication
- guardrail workflows for repository layout and ownership rules

## Alignment Rules

- local make targets must mirror CI gate composition
- path filters must reflect real ownership boundaries
- failure messages must name owning surface and remediation path

## Pipeline Ownership Rule

Every required workflow must declare:

- owning maintainer role
- owning handbook page for remediation guidance
- escalation path when the owner is unavailable

## Reading Rule

Use this page when the question is whether CI is enforcing the same contract the
repository expects locally. Move to GitHub workflows or Repository Gates once
the mismatch is narrowed to one workflow or gate family.

## Code Anchors

- `.github/workflows/`
- `makes/gh.mk`
- `crates/bijux-dev/src/suites/repo.rs`

## Next Reads

- [gh-workflows](../gh-workflows/index.md)
- [makes](../makes/ci-targets.md)
- [Repository Gates](repository-gates.md)
- [Release Operations](release-operations.md)
- [Change Control](../governance/change-control.md)
