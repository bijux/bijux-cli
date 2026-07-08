---
title: Test Strategy
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Test Strategy

Use this page when you need to know which test lane is supposed to prove a CLI
claim, and which lane is only incidental coverage.

`bijux-cli` does not rely on one giant bucket of tests. It layers focused proof
so parser laws, command behavior, ownership boundaries, and release surfaces
can fail for the right reasons.

## Test Layers

- unit tests in source modules for local behavior
- routing suites for parser normalization and route laws
- integration suites for command end-to-end behavior
- architecture suites for boundary and ownership guarantees
- release/contract tests for packaging and schema confidence

## What Each Layer Should Prove

| Layer | What it should catch |
| --- | --- |
| unit | local logic regressions before they become command-surface regressions |
| routing | path normalization, alias handling, help routing, and suggestion drift |
| integration | real command behavior, plugin lifecycle flow, and output semantics |
| architecture | ownership violations and accidental boundary collapse |
| release and contract | packaging, schema, and publication-surface dishonesty |

## Slow-Test Policy

- tests above 10 seconds should be marked and excluded from default fast gates
- slow suites should run in dedicated gates or explicit invocations
- default `make test` should prioritize quick contract feedback

## Code Anchors

- `crates/bijux-cli/tests/routing/`
- `crates/bijux-cli/tests/integration/`
- `crates/bijux-cli/tests/architecture/`
- `makes/`

## Test Strategy Rules

- new user-visible behavior requires targeted tests in the owning layer
- snapshot changes need explicit review comments and rationale
- failing architecture tests block merges even if local unit tests pass

## Reader Shortcut

If a behavioral claim is defended only by a broad workspace pass and not by the
owning CLI lane, the evidence is weaker than it looks. The right lane should
fail when the contract it owns drifts.

## Continue Reading

- [Invariants](invariants.md)
- [Change Validation](change-validation.md)
- [Risk Register](risk-register.md)
