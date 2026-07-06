---
title: Spec To Code And Test Ownership
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-06
---

# Spec To Code And Test Ownership

Repository specs are only trustworthy when each one has a visible owning code
surface and verifying tests.

## Ownership Rules

- spec pages must point to owning crate or package surfaces
- contract pages must link to verifying tests
- claims about capabilities, replay, planner behavior, and runtime trust must
  stay aligned with executable proof
- generated reports may summarize proof, but they do not replace owning specs

## Review Questions

1. which code surface owns this behavior?
2. which tests fail if the behavior drifts?
3. which report or handbook page explains it to a reader?

## Next Reads

- [Documentation Standards](documentation-standards.md)
- [Documentation Governance Alignment](documentation-governance-alignment.md)
- [Testing and Validation](testing-and-validation.md)
