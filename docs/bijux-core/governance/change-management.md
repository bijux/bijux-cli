---
title: Change Management
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Change Management

Change management ensures behavior changes are reviewed with explicit scope,
verification evidence, and documentation updates.

## Visual Summary

```mermaid
flowchart LR
    proposal[change proposal] --> scope[classify ownership and impact]
    scope --> implement[implement in owning crate]
    implement --> validate[validate with tests and contracts]
    validate --> document[update docs and risks]
    document --> merge[merge decision]
```

## Required Steps

1. identify owning crate and handbook section
2. classify impact as internal, interface, or compatibility-sensitive
3. run targeted and cross-surface validation
4. update docs, risks, and decision records where applicable
5. merge only with reviewable evidence attached

## Evidence Rules

- assertions without tests or contract checks are incomplete
- compatibility-sensitive changes require explicit migration notes
- docs updates belong in the same change set as behavior updates

## Code Anchors

- `crates/bijux-dev/src/suites/`
- `crates/bijux-dev/src/commands/contract_governance.rs`
- `crates/bijux-dev/src/commands/docs_governance.rs`

## Next Reads

- [Testing and Validation](testing-and-validation.md)
- [Decision Record Policy](decision-record-policy.md)
- [Risk and Exceptions](risk-and-exceptions.md)
