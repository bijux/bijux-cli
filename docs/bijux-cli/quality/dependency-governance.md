---
title: Dependency Governance
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Dependency Governance

Use this page when a dependency update looks routine but might change what the
CLI accepts, prints, serializes, or promises.

Dependency governance matters because `bijux-cli` leans on parser, schema,
serialization, and compatibility libraries that can shift user-visible
behavior even when local code changes are small.

## Governance Focus

- parser grammar and help behavior (`clap`)
- payload and schema serialization (`serde`, `serde_json`, `schemars`)
- compatibility range semantics (`semver`)
- error typing and propagation (`thiserror`, `anyhow`)

## Code Anchors

- `crates/bijux-cli/Cargo.toml`
- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/contracts/schema.rs`
- `crates/bijux-cli/src/contracts/plugin.rs`
- `crates/bijux-cli/tests/routing/`

## What Reviewers Should Check

| Dependency surface | Why it is risky |
| --- | --- |
| parser and help libraries | route grammar, help text, and argv interpretation may drift |
| schema and serialization libraries | machine-readable outputs can change shape or ordering |
| semver logic | compatibility acceptance and plugin range checks can loosen or tighten unexpectedly |
| error and propagation libraries | diagnostics may change wording or classification in contract-facing flows |

## Governance Rules

- no dependency bumps without targeted test evidence
- document behavior changes caused by dependency upgrades
- avoid broad upgrade bundles that hide root-cause regressions
- keep dependency decisions auditable in commit and review history

## Reader Shortcut

If a dependency change alters command grammar, payload shape, or compatibility
range behavior, the dependency did not stay internal. Review it like a contract
change, because for users it effectively is one.

## Continue Reading

- [Dependencies and Adjacencies](../foundation/dependencies-and-adjacencies.md)
- [Change Validation](change-validation.md)
- [Risk Register](risk-register.md)
