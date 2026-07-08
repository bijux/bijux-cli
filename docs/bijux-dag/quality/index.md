---
title: DAG Quality
audience: maintainers
type: section-index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
---

# DAG Quality

DAG quality defines the proof required for behavior changes, compatibility
claims, and operational trust.

## Quality Goals

- keep replay and diff semantics stable across change
- require evidence-backed validation before release
- maintain explicit risk and limitation records
- align docs with real command and code behavior

## Core Quality Pages

- [Test Strategy](test-strategy.md)
- [Change Validation](change-validation.md)
- [Invariants](invariants.md)
- [Definition of Done](definition-of-done.md)
- [Review Checklist](review-checklist.md)

## Governance Pages

- [Dependency Governance](dependency-governance.md)
- [Risk Register](risk-register.md)
- [Known Limitations](known-limitations.md)
- [Documentation Standards](documentation-standards.md)

## Reading Rule

Use this section when the question is about what evidence must exist before DAG
behavior can be trusted. Move back to Operations when the next question is how
to run the checks rather than what they must establish.
