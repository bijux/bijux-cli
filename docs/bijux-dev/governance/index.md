---
title: Dev Governance
audience: maintainers
type: section-index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Dev Governance

This section defines maintainership governance for `bijux-dev` command behavior,
quality expectations, and policy controls.

## Visual Summary

```mermaid
flowchart LR
    ownership[ownership model] --> quality[quality policy]
    quality --> tests[test policy]
    tests --> change[change control]
    change --> contracts[contract governance]
    contracts --> limits[known limitations]
```

## Pages In This Section

- [Ownership Model](ownership-model.md)
- [Quality Policy](quality-policy.md)
- [Test Policy](test-policy.md)
- [Change Control](change-control.md)
- [Contract Governance](contract-governance.md)
- [Dependency Governance](dependency-governance.md)
- [Documentation Standard](documentation-standard.md)
- [Security and Secrets](security-and-secrets.md)
- [Known Limitations](known-limitations.md)
