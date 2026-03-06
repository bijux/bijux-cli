# Testing strategy by crate

## Core crate

- Structural parser and validation behavior in `crates/bijux-dag-core/tests`.
- Compatibility fixtures in `crates/bijux-dag-core/tests/compat/v0.1`.

## Runtime crate

- Determinism, replay, cache, and failure behavior in `crates/bijux-dag-runtime/tests`.

## App crate

- Command contract and output behavior in `crates/bijux-dag-app/tests`.

## CLI crate

- Surface, JSON schema, and exit-code contract checks in `crates/bijux-dag-cli/tests`.

## Dev crate

- Repository architecture, policy, suite orchestration, and release guardrails in `crates/bijux-dev-dag/tests`.
