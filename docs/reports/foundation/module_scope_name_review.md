# Module Scope Name Review

generated_from: `crates/*/src module inventory`

## Review Goal

Identify module names that imply platform/control-plane scope where DAG-kernel naming should apply.

## Current Result

- no crate module rename executed in this pass.
- broad-scope names remain quarantined through runtime broad-surface ownership policy.

## Rename Backlog

- deferred until owner-repo migration of quarantined runtime surfaces completes.
