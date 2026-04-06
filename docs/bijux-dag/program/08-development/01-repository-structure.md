# Repository Structure

This guide explains how to navigate the repository by responsibility so contributors can make changes without crossing boundaries accidentally.

## Conceptual map

Use this map first, then locate concrete paths:
- runtime behavior: execution engine, scheduling, identity-bearing semantics,
- command surface: CLI/operator-facing entry points,
- specification surface: normative contracts and semantic vocabulary,
- operations surface: CI/security/trust guidance,
- support tooling: scripts and automation that assist development.

## Product-path versus support-path code

Product-path code:
- defines runtime behavior users rely on,
- affects run/artifact/replay/diff semantics,
- requires aligned updates in specs and user docs when behavior changes.

Support-path code:
- enables development, CI, or maintenance workflows,
- should not silently redefine runtime contracts,
- can evolve independently when behavior contracts remain unchanged.

Governance/internal materials:
- help maintainers coordinate work,
- are not authoritative runtime semantics,
- must not conflict with specification docs.

## Fast orientation workflow

When starting a change:
1. identify the contract affected (spec or user-facing behavior).
2. locate owning implementation area in crates.
3. locate lane-specific tests proving that contract.
4. apply docs/code/test changes in the same conceptual surface.

This sequence prevents scope widening and misplaced edits.

## Boundary rule

If a change touches multiple conceptual surfaces, document why each surface is required; otherwise split the change.

## Guarantees

- Contributors can locate ownership by responsibility, not guesswork.
- Product-path changes are explicitly separated from support-path changes.

## Non-guarantees

- Exact file/module layout permanence.
- Automatic correctness from directory placement alone.

## Next reading

- [Crate architecture](../05-system-architecture/02-crate-architecture.md)
- [Testing strategy](02-testing-strategy.md)
- [Contributing](04-contributing.md)
