# Crate taxonomy v2

## Purpose

Define stable workspace crate ownership and dependency boundaries for the current foundation scope.

## One-sentence responsibilities

- `bijux-dag-core`: DAG schema, parsing, canonicalization, validation, and deterministic semantic graph logic.
- `bijux-dag-artifacts`: run artifact models, persistence services, integrity proofs, and lifecycle policy helpers.
- `bijux-dag-runtime`: execution engine, scheduler behavior, policy enforcement, replay semantics, and runtime diagnostics.
- `bijux-dag-testkit`: shared deterministic test fixtures, builders, and assertion helpers for workspace crates.
- `bijux-dag-app`: application orchestration services, command response modeling, and user-facing render flows.
- `bijux-dag-cli`: thin process entrypoint that delegates to app command surfaces.
- `bijux-dev-dag`: repository governance control-plane, suite orchestration, and release verification automation.

## Dependency boundary

Allowed workspace edges are defined in:

- `configs/policy/crate_taxonomy_v2.json`

`bijux-dev-dag` enforces this policy through taxonomy guardrail tests.

## Taxonomy decisions

- app remains one crate for this scope.
- artifacts remains one crate with explicit internal sub-boundaries.
- planning remains in core with runtime bridge consumption.
- runtime remains one crate after contraction and policy freeze.
- testkit remains shared support for tests.
- container/remote/batch stay modeled in runtime as future execution boundaries.

## Stability rule

This taxonomy is in frozen mode. New workspace crates are blocked until this document and `crate_taxonomy_v2` policy are explicitly revised together.
