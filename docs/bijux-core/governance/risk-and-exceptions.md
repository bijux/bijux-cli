---
title: Risk and Exceptions
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Risk and Exceptions

Risk and exception policy keeps urgent decisions explicit instead of silently
weakening quality gates.

## Visual Summary

```mermaid
flowchart LR
    risk[identify risk] --> classify[classify severity and scope]
    classify --> exception[exception request when needed]
    exception --> mitigate[mitigation and expiry]
    mitigate --> verify[follow-up verification]
```

## Risk Categories

- compatibility drift risk
- release reliability risk
- governance and documentation drift risk
- dependency and supply-chain risk

## Exception Rules

- every exception needs owner, scope, and expiration date
- mitigations must be concrete and testable
- expired exceptions must be closed or renewed with evidence

## Exception Record Template

Use this structure for every exception request:

- `owner`: accountable maintainer
- `scope`: exact affected crate/docs surface
- `reason`: why the standard gate cannot pass now
- `mitigation`: immediate controls while exception is active
- `expiry`: specific date for revalidation or removal

## Code Anchors

- `crates/bijux-dev/src/commands/ops.rs`
- `crates/bijux-dev/src/suites/repo.rs`
- `crates/bijux-dev/src/suites/release.rs`

## Next Reads

- [Testing and Validation](testing-and-validation.md)
- [Change Management](change-management.md)
- [Architecture Risks](../architecture/architecture-risks.md)
