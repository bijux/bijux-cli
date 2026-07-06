---
title: Container Execution Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Container Execution Contract

Container execution in `bijux-dag` is a governed local execution mode with
typed contract validation, path normalization, and explicit environment
shaping rules.

## Scope

This contract covers the typed container execution descriptor, container mount
and output-path validation, GPU runtime argument shaping, local-to-container
path mapping, and environment isolation behavior exercised by
`crates/bijux-dag-runtime/tests/container_execution_contracts.rs`.

## Required container contract fields

The runtime container contract must declare:

- a non-empty `image`
- a non-empty `command`
- at least one mount
- declared output paths that remain relative to the run boundary

Image literals are validated as image names, not shell options.

## Path and mount rules

- mount mappings must preserve a deterministic local-to-container path rewrite
- declared output paths must reject traversal such as `../escape`
- normalized relative container paths are part of the governed contract

## Environment and runtime rules

- container environment isolation is governed by allowlist and denylist rules
- GPU runtime arguments are supported only for recognized engines
- unsupported engines must reject GPU requests explicitly

## Related tests

- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`

## Versioning and change policy

Any incompatible change to container contract fields, path normalization,
environment isolation semantics, or GPU runtime argument behavior must update
this contract and the linked runtime tests in the same change.
