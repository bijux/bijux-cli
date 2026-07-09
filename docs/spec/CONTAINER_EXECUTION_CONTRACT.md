---
title: Container Execution Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Container Execution Contract

Container execution in `bijux-dag` is an implemented local execution mode with
typed contract validation, explicit mount boundaries, path normalization, and
environment shaping rules.

## Scope

This contract covers the typed container execution descriptor, mounted input
materialization, declared output collection, timeout handling, stdout/stderr
capture, image identity tracing, container network-mode shaping, GPU runtime
argument shaping, local-to-container path mapping, and environment isolation
behavior exercised by:

- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/adapter_runtime_contracts.rs`

## Required container contract fields

The runtime container contract must declare:

- a non-empty `image`
- a non-empty `command`
- at least one mount
- declared output paths that remain relative to the run boundary

Image literals are validated as image names, not shell options.

## Path and mount rules

- mount mappings must preserve a deterministic local-to-container path rewrite
- upstream materialized inputs must be mounted under `/bijux/node/inputs`
- declared outputs must be collected from `/bijux/node/outputs`
- node work state must stay under `/bijux/node/work`
- declared output paths must reject traversal such as `../escape`
- normalized relative container paths are part of the governed contract

## Environment and runtime rules

- container environment isolation is governed by allowlist and denylist rules
- stable execution requires digest-pinned image references by default; tag-only
  references are rejected unless the operator explicitly allows unpinned images
- timeout termination must preserve partial stdout and stderr for operator
  inspection
- node traces must record the declared image reference and any discovered image
  digest or engine version evidence
- deny-network must map to a concrete engine flag when the selected engine can
  enforce it
- GPU runtime arguments are supported only for recognized engines
- unsupported engines must reject GPU requests explicitly

## Versioning and change policy

Any incompatible change to container contract fields, path normalization,
environment isolation semantics, or GPU runtime argument behavior must update
this contract and the linked runtime tests in the same change.

## Related tests

- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/adapter_runtime_contracts.rs`
- `crates/bijux-dag-app/tests/container_workflow_contract.rs`
