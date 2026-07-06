---
title: Config Policy Determinism Report
audience: maintainer
type: report
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Config Policy Determinism Report

## Purpose

This report records the repository surfaces that currently prove deterministic
config precedence and operator-visible policy evaluation behavior.

## Guarded surfaces

- precedence contract: `docs/spec/CONFIG_PRECEDENCE_CONTRACT.md`
- policy trace contract: `docs/spec/POLICY_EVALUATION_TRACE.md`
- interface overview: `docs/bijux-dag/interfaces/configuration-surface.md`
- command implementation: `crates/bijux-dag-app/src/commands/config_resolution.rs`
- config precedence tests: `crates/bijux-dag-app/tests/config_precedence_contract.rs`
- config validation tests: `crates/bijux-dag-app/tests/config_validation_contract.rs`
- effective command tests: `crates/bijux-dag-app/tests/config_effective_command_contract.rs`
- runtime policy tests: `crates/bijux-dag-runtime/tests/security_model_contracts.rs`
- trust property: `tp_config_policy_determinism`

## Determinism boundary

- merge order is `CLI > explicit config file > environment > defaults`
- explicit config parse and shape failures are blocking, not advisory
- `dag config show-effective` and `dag policy show-effective` expose the merged
  result before execution
- policy traces explain effective decisions rather than hiding precedence

## Residual limits

- environment-derived config remains process-local state and therefore must be
  surfaced explicitly when effective config is inspected
- determinism here governs config and policy resolution, not full execution
  reproducibility on its own
