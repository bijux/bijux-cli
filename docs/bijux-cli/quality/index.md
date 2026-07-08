---
title: Quality
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# CLI Quality

The quality section defines how `bijux-cli` changes are validated, reviewed,
and documented before release.

## Quality Scope

- test layering and execution expectations
- invariant checks that protect command contracts
- review checklist criteria for safe merges
- documentation and evidence standards
- known limitations and risk-tracking practices

## Code Anchors

- `crates/bijux-cli/tests/`
- `crates/bijux-cli/src/contracts/`
- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `makes/docs.mk`

## Pages In This Section

- [Test Strategy](test-strategy.md)
- [Invariants](invariants.md)
- [Review Checklist](review-checklist.md)
- [Documentation Standards](documentation-standards.md)
- [Definition of Done](definition-of-done.md)
- [Dependency Governance](dependency-governance.md)
- [Change Validation](change-validation.md)
- [Known Limitations](known-limitations.md)
- [Risk Register](risk-register.md)

## Reading Rule

Use this section when the question is about what proof is required before a
change can be trusted. Move back to Operations when the next question is how to
run the checks rather than what the checks must prove.
