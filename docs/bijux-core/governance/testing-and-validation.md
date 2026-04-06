---
title: Testing and Validation
audience: mixed
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Testing and Validation

Testing and validation policy aligns workspace checks, program suites, and docs
quality gates into one coherent evidence model.

## Visual Summary

```mermaid
flowchart TD
    unit[targeted crate tests] --> integration[program integration suites]
    integration --> contracts[contract and schema checks]
    contracts --> docs[docs-check and link hygiene]
    docs --> readiness[release readiness evidence]
```

## Validation Layers

- crate-level unit and integration tests in owning packages
- contract suites for public behavior and schema stability
- maintainer governance suites for layout and policy contracts
- documentation checks for structure, links, and publishability

## Required Commands

```bash
cargo test --workspace
make test
make docs-check
cargo run -q -p bijux-dev --bin bijux-dev-cli -- verify
```

## Code Anchors

- `makes/rust.mk`
- `makes/docs.mk`
- `crates/bijux-dev/src/suites/test.rs`
- `crates/bijux-dev/src/suites/docs.rs`

## Next Reads

- [Release and Versioning](release-and-versioning.md)
- [Compatibility and Schema](compatibility-and-schema.md)
- [Core Architecture](../architecture/index.md)
