---
title: Module Surface Lanes
audience: mixed
type: explanation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Module Surface Lanes

`bijux-core` classifies crate module exports by surface lane so public modules
and internal modules do not drift into ambiguous ownership.

The governing contract for this page is
`contracts/foundation/module_surface_lanes.v1.json`.

## Lane Meanings

| Lane | Meaning |
| --- | --- |
| stable | public module surface intended for durable downstream use |
| experimental | public module surface exposed deliberately but outside the stable compatibility lane |
| simulated | public module surface reserved for modeled or gated repository workflows |
| private | default lane for non-public modules that stay internal to the crate |

## Reading Rules

- Treat `private` as the default lane unless a module is explicitly listed in
  the contract's public surface entries.
- Keep `stable`, `experimental`, and `simulated` exports disjoint.
- When a top-level `pub mod` changes, update the contract and the related
  contract tests in the same change set.
- Do not use module names or lane descriptions that depend on temporary
  delivery order or roadmap shorthand.

## Verification

- `contracts/foundation/module_surface_lanes.v1.json`
- `crates/bijux-dev/tests/foundation_module_surface_contracts.rs`
- the library entrypoint owned by each workspace crate

## Next Reads

- [Documentation System](documentation-system.md)
- [Package Boundary](package-boundary.md)
- [Domain Language](domain-language.md)
