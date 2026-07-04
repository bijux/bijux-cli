---
title: Review Checklist
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
---

# Review Checklist

Use this checklist during DAG review to prevent silent compatibility, evidence,
or documentation regressions.

## Visual Summary

```mermaid
flowchart TD
    review[Review starts] --> ownership[Correct ownership?]
    ownership --> architecture[Architecture still honest?]
    architecture --> contracts[Contracts and semantics clear?]
    contracts --> tests[Tests adequate?]
    tests --> docs[Docs and links aligned?]
    docs --> approve[Approve]
    ownership --> request[Request changes]
    architecture --> request
    contracts --> request
    tests --> request
    docs --> request
```

## Required Checks

- change stays within clear crate/module ownership boundaries
- tests cover modified behavior and relevant contracts
- replay/diff semantics and reason-code meanings remain explicit
- artifact evidence expectations remain intact and verifiable
- docs links, examples, and code anchors match current repository state
- touched limitation records keep stable ids and include affected surface,
  impact, workaround, planned fix, and release target fields
- touched risk records keep stable ids and include severity, affected
  component, current status, mitigation, and release decision fields

## Structural Checks

- `docs/bijux-dag` contains exactly five section directories
- each section contains exactly ten markdown files
- no references to removed nested `program/*` docs remain

## Next Reads

- [Definition of Done](definition-of-done.md)
- [Documentation Standards](documentation-standards.md)
- [DAG Operations](../operations/index.md)
