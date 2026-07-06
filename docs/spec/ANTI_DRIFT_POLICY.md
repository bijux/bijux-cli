---
title: Anti Drift Policy
audience: mixed
type: spec
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-06
---

# Anti Drift Policy

This policy defines how `bijux-core` prevents release-facing repository drift
between documented behavior, contract surfaces, CLI shape, executable tests,
and machine-readable release checks.

## Scope

This policy governs repository-level drift detection for docs drift, schema
drift, contract drift, cli drift, fixture drift, benchmark drift, and
dependency drift.

## Same-change alignment rule

The same-change alignment rule is mandatory for release-facing surfaces:

- docs drift must be corrected in the same change as the behavior change
- schema drift must update both schema references and the owning executable
  surfaces
- contract drift must update the contract document and its related tests
- cli drift must update help text, examples, and route inventory together
- fixture drift must update governed fixtures and their docs references together
- benchmark drift must keep evidence links explicit when benchmark-oriented claims are
  introduced

## Required checks

- `cli-freeze`
- `docs-schema-ref`
- `docs-contract-ref`
- `contract-test-links`
- `docs-coverage`
- `versioning-compatibility`
- benchmark-claim governance gate

## Related tests

- `crates/bijux-dev/src/commands/ops.rs`
- `.github/pull_request_template.md`

## Versioning and change policy

Any incompatible change to drift classes, required repo checks, or alignment
rules must update this policy and the linked governance surfaces in the same
change.
