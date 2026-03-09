# ADR: Stable Operator Surface

## Status

Accepted

## Context

Operator command surfaces accumulated overlapping outputs and multiple ways to answer similar operational questions. This increased cognitive load and made stable usage harder to explain.

## Decision

1. Maintain a compact canonical operator command set for default workflows.
2. Keep detailed output opt-in while preserving concise defaults.
3. Keep JSON output available for core automation-facing commands.
4. Treat experimental or modeled terms as non-default and clearly marked.
5. Enforce operator surface stability through dedicated verification suite and contracts.

## Consequences

- Operator workflows become easier to teach and automate.
- Redundant command stories are reduced without removing needed diagnostic depth.
- Surface drift is detected via contract and snapshot tests.

## Enforcement

- `configs/suites/operator_surface_verification.json`
- `crates/bijux-dev-dag/tests/operator_surface_guarantees_contracts.rs`
