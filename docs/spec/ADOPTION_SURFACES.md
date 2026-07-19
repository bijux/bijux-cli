---
title: Adoption Surfaces
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Adoption Surfaces

`bijux-dag` adoption depends on a small set of operator-facing documents,
fixtures, and executable release checks that make the local product boundary
understandable without repository archaeology.

## Scope

This specification governs installation, CI integration, first-hour operator
flow, support framing, trust boundaries, and release verification entrypoints
for `v0.4.0`.

## Required adoption bundle

- installation path: `docs/bijux-dag/operations/installation-and-setup.md`
- CI path: `docs/bijux-dag/operations/ci-integration.md`
- first-hour walkthrough: `docs/bijux-dag/operations/first-hour-with-bijux-dag.md`
- support framing: `docs/bijux-dag/interfaces/support-matrix.md`
- trust boundary: `docs/bijux-dag/operations/trust-boundaries.md`
- release verification: `docs/spec/RELEASE_BINARY_VERIFICATION.md`
- starter fixture: `evidence/dag/authoring/examples/minimal_consumer.dag.json`
- testkit fixture README:
  `crates/bijux-dag-testkit/fixtures/minimal_consumer/README.md`

## Product-boundary rule

Adoption surfaces must describe the implemented local DAG runtime, its visible
stable commands, and its explicit simulated or future boundaries. They must not
present Kubernetes, HPC, or promoted remote coordination as first-hour
product promises.

## Related tests

- `crates/bijux-dev/src/commands/ops.rs`
- `docs/spec/RELEASE_BINARY_VERIFICATION.md`

## Versioning and change policy

Any incompatible change to installation commands, support framing, or adoption
entrypoints must update this specification and the linked docs in the same
change.
