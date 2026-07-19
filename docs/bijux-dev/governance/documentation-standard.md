---
title: Documentation Standard
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Documentation Standard

Use this page when you are writing or reviewing maintainer documentation and
need the practical quality bar, not a style exercise.

The standard exists because maintainer docs are often read under pressure:
while diagnosing a broken gate, preparing a release, or checking whether a
governance claim is still true. A page that is technically correct but hard to
scan is still bad documentation.

## What Good Maintainer Docs Must Do

- use canonical frontmatter on every page
- keep links inside the active handbook tree and route readers to the owning
  page instead of a vague section index
- connect policies to real commands, files, contracts, or tests
- avoid placeholder language without operational meaning
- make it clear what is real today versus what is only guidance or future work

## Writing Rules That Matter

- use direct declarative language when behavior is required
- avoid hand-wavy phrasing that cannot be checked against commands or files
- prefer short sentences that map to a maintainer action or decision
- keep incident and remediation guidance ordered and concrete
- explain the boundary of a page before diving into details

## What Readers Should Be Able To Learn Fast

- what this page is for
- what is required versus optional
- which command, file, or contract proves the claim
- where to go next if the question belongs to another handbook

## Alignment Rules

- `docs/bijux-core` and `docs/bijux-dev` follow parallel section patterns
- maintainer docs avoid duplicating CLI and DAG product semantics
- MkDocs nav remains synchronized with filesystem layout

## Common Documentation Failures

- pages that describe how documentation should behave instead of explaining the
  repository surface itself
- diagrams with no operational takeaway
- policies with no command, contract, or file anchor
- links that force readers to hunt across handbooks for the real owner

## Code Anchors

- `mkdocs.yml`
- `docs/bijux-dev/`
- `makes/docs.mk`

## Continue Reading

- [Documentation Operations](../operations/docs-operations.md)
- [Core Documentation Standards](../../bijux-core/governance/documentation-standards.md)
- [Known Limitations](known-limitations.md)
