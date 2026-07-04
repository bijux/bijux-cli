---
title: gh-workflows
audience: mixed
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Dev GitHub Workflows

This section maps the GitHub Actions entrypoints in `bijux-core`. Use it to
identify which workflow owns a failure, release gate, or automation path before
debugging jobs directly in CI logs.

## Pages In This Section

- [ci](ci.md)
- [release-validation](release-validation.md)
- [deploy-docs](deploy-docs.md)
- [release-crates](release-crates.md)
- [release-pypi](release-pypi.md)
- [release-github](release-github.md)
- [bijux-canon](bijux-canon.md)

## Workflow Ownership Map

| Workflow | Owns | Enter Here When |
| --- | --- | --- |
| [ci](ci.md) | Repository validation gates and baseline verification | pull request checks fail or mandatory CI signals regress |
| [release-validation](release-validation.md) | Release-candidate verification from committed `HEAD` | release checks fail on formatting, packaging, publish dry runs, or smoke coverage |
| [deploy-docs](deploy-docs.md) | Documentation publishing and site deployment flow | docs site output, publish triggers, or deploy permissions fail |
| [release-crates](release-crates.md) | Rust crate release orchestration | crates release process, publish ordering, or registry upload issues appear |
| [release-pypi](release-pypi.md) | Python release orchestration | PyPI publication, packaging metadata, or python release gates fail |
| [release-github](release-github.md) | GitHub release record and release artifacts | release notes, release tagging, or artifact publication on GitHub breaks |
| [bijux-canon](bijux-canon.md) | Cross-repo canon integration workflow | canon handoff integration or shared validation contracts fail |

## Reading Rule

Start with workflow ownership first, then inspect job-level behavior. If a
failure spans multiple workflows, treat it as a contract boundary issue rather
than patching one workflow in isolation.
