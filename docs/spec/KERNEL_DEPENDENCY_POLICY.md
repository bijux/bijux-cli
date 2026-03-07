# Kernel Dependency Policy

## Purpose

Defines allowed and forbidden dependency classes for kernel-owned code.

## Allowed classes

- `serde`, `serde_json`
- hashing and deterministic canonicalization dependencies
- core/runtime artifact and cache dependencies
- deterministic scheduler/planner dependencies

## Forbidden classes

- CLI and command-line parsing dependencies in kernel crates
- dev governance/control-plane dependencies in kernel crates
- report-generation and evidence-report formatting dependencies in kernel crates

## Machine policy source

- `configs/policy/kernel_dependency_policy.json`

## Related tests

- `crates/bijux-dev-dag/tests/kernel_boundary_contracts.rs`
