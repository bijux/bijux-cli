---
title: Testing and Validation
audience: mixed
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Testing and Validation

This page explains how `bijux-core` turns changes into reviewable evidence.

The repository uses several layers of verification, but the idea is not
complicated: each layer answers a different kind of risk before a change moves
forward.

## Validation Flow

```mermaid
flowchart TD
    change["proposed change"] --> unit["crate and package tests"]
    change --> contracts["contract and schema checks"]
    change --> docs["docs and link checks"]
    unit --> readiness["merge or release evidence"]
    contracts --> readiness
    docs --> readiness
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
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
```

## Minimum Review Evidence

- failing-to-passing command outputs for affected surfaces
- contract-test confirmation for schema-sensitive changes
- docs-check output for documentation-affected changes
- explicit note for any skipped check with owner and follow-up date

## Reading Rule

Stay on this page when the question is what kind of proof a change needs. Move
to the package or operations handbooks when the proof model is clear and the
next question is how to run or debug a specific check.

## Code Anchors

- `makes/rust.mk`
- `makes/docs.mk`
- `crates/bijux-dev/src/suites/test.rs`
- `crates/bijux-dev/src/suites/docs.rs`

## Next Reads

- [Release and Versioning](release-and-versioning.md)
- [Compatibility and Schema](compatibility-and-schema.md)
- [Core Architecture](../architecture/index.md)
