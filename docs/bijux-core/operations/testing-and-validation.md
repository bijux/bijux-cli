---
title: Testing and Validation
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Testing and Validation

Repository validation combines package-local tests, root make gates, and docs
checks into one reviewable evidence model.

```mermaid
flowchart TB
    confidence[Repository confidence]
    confidence --> package_tests[package test layers]
    confidence --> root_checks[root make gates]
    confidence --> contracts[contract/schema checks]
    confidence --> docs_checks[docs and navigation checks]

    package_tests --> evidence[reviewable evidence set]
    root_checks --> evidence
    contracts --> evidence
    docs_checks --> evidence
```

## Canonical Commands

```bash
make test
make dag-test
make docs-check
```

## Validation Rule

Run the owning package checks and the root checks that prove the repository
still publishes, routes, and documents the change honestly.

## Next Reads

- [Review Expectations](review-expectations.md)
- [Change Management](change-management.md)
- [Core Testing and Validation](../governance/testing-and-validation.md)
