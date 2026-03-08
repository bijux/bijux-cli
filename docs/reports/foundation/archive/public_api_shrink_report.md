# Public API Shrink Report

Date: 2026-03-07

## Objective

Track shrink progress of externally exposed surfaces toward kernel-stable API boundaries.

## Current status

- Core and runtime retain large historic public surfaces.
- Kernel boundary contract now defines the stable target categories.
- Modeled/future platform declarations are documented as non-kernel and non-release-proof.

## Shrink plan

1. Keep kernel API categories stable and test-backed.
2. Move or demote non-kernel public exports in incremental waves.
3. Convert internal-only exposures to `pub(crate)` when no external consumer requires `pub`.
4. Keep release and evidence claims tied to kernel-stable surfaces only.

## Guardrails

- Kernel dependency policy checks.
- Runtime overreach policy checks.
- Release evidence linkage checks.
