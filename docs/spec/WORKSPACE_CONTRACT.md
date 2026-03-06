# Workspace contract

This document defines crate responsibilities and allowed dependency directions.

## Crate responsibilities

- `bijux-dag-core`: DAG model, parsing, canonicalization, validation, fingerprinting, and topology algorithms. Pure logic only.
- `bijux-dag-artifacts`: artifact/run manifest data models and artifact persistence contracts.
- `bijux-dag-runtime`: execution planning/runtime, scheduling flow, adapter invocation boundaries, policy enforcement, and trace emission.
- `bijux-dag-app`: application orchestration commands and structured output rendering.
- `bijux-dag-cli`: binary wiring and process-level error mapping only.
- `bijux-dev-dag`: repository governance, contract checks, release verification orchestration.

## Allowed crate dependency directions

- `bijux-dag-core`: may not depend on runtime/app/cli/dev crates.
- `bijux-dag-artifacts`: may depend on core models only when required by artifact contracts.
- `bijux-dag-runtime`: may depend on core and artifacts.
- `bijux-dag-app`: may depend on core, artifacts, runtime.
- `bijux-dag-cli`: may depend on app only.
- `bijux-dev-dag`: must not depend on runtime internals or app runtime orchestration internals.

## Enforcement

Boundary policy is enforced by `configs/policy/dependency_rules.json` through `bijux-dev-dag dep-guard`.
