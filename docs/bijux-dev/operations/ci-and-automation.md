---
title: CI and Automation
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# CI and Automation

Use this page when a change is green in one place and you need to know whether
the repository is actually enforcing the same proof locally and in hosted
automation.

`bijux-core` depends on that alignment because a green PR is not meaningful if
maintainers cannot reproduce the same gate composition before the push.

## Automation Scope

- build, test, and contract workflows
- docs build and publish workflows
- release workflows for crate and package publication
- guardrail workflows for repository layout and ownership rules

## Alignment Rules

- local make targets must mirror CI gate composition
- release-candidate verification must use the same committed-`HEAD` suite locally and in CI
- path filters must reflect real ownership boundaries
- failure messages must name owning surface and remediation path

## What Reviewers Should Check

| Surface | Why it matters |
| --- | --- |
| local vs CI gate composition | mismatched lanes create false confidence |
| path filters and workflow triggers | skipped automation can hide broken ownership boundaries |
| failure output quality | a red workflow must point to the real owner and next action |

## Pipeline Ownership Rule

Every required workflow must declare:

- owning maintainer role
- owning handbook page for remediation guidance
- escalation path when the owner is unavailable

## Reader Shortcut

If a workflow is green only because local and hosted automation are checking
different things, the automation is lying. Fix the alignment before trusting
the result.

## Code Anchors

- `.github/workflows/`
- `makes/gh.mk`
- `crates/bijux-dev/src/suites/repo.rs`

## Continue Reading

- [gh-workflows](../gh-workflows/index.md)
- [makes](../makes/ci-targets.md)
- [Repository Gates](repository-gates.md)
- [Release Operations](release-operations.md)
- [Release Validation Suite](release-validation-suite.md)
- [Change Control](../governance/change-control.md)
