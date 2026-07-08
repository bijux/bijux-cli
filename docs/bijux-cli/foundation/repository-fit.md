---
title: Repository Fit
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Repository Fit

`bijux` lives inside `bijux-core`, but it is not the whole repository. Use this
page when you need to know where the CLI story ends and where root,
workflow, or maintainer documentation should take over.

That boundary matters because the repository now carries multiple products and
maintenance surfaces. Readers should not have to guess whether a statement is
about the command runtime itself, the DAG stack, or repository governance.

## What This Handbook Owns

| Surface | What belongs here |
| --- | --- |
| command runtime | argv parsing, route resolution, built-in features, plugin handling, output contracts |
| CLI product behavior | what `bijux` does directly for users and automation |
| routed integrations | how the CLI discovers or delegates to known tools without claiming to own them |

## What Belongs Somewhere Else

| Surface | Where readers should go |
| --- | --- |
| DAG semantics and workflow evidence | `docs/bijux-dag/` |
| repository-wide documentation rules and publication structure | root docs and handbook indexes |
| maintainer automation, CI governance, and internal release mechanics | `docs/bijux-dev/` |

## Why The Boundary Matters

- It prevents CLI documentation from making claims about DAG behavior it does
  not own.
- It keeps maintainer-only control surfaces out of user-facing product pages.
- It makes route delegation understandable without pretending delegation is the
  same thing as ownership.

## Code Anchors

- `crates/bijux-cli/src/contracts/product_mount.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/src/interface/cli/dispatch/delegation.rs`
- `contracts/official_product_namespace_registry.json`

## Reader Rules

- CLI pages describe `bijux` runtime behavior and contracts only
- DAG semantics are documented in the DAG handbook
- maintainer workflows and CI orchestration are documented in the dev handbook
- root docs own cross-package scope, layout, and publication rules

## Continue Reading

- [Dependencies and Adjacencies](dependencies-and-adjacencies.md)
- [Integration Seams](../architecture/integration-seams.md)
- [Deployment Boundaries](../operations/reference/deployment-boundaries.md)
