---
title: Extensibility Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Extensibility Contract

`bijux-dag` supports a narrow extension surface with explicit stability levels,
typed descriptors, and failure-isolation requirements.

## Scope

This contract covers extension descriptor shape, extension-point stability,
registration conflict handling, compatibility checks, discovery inventory, and
internal hook promotion rules implemented in
`crates/bijux-dag-runtime/src/internal/ext/extension_catalog.rs`.

## Supported extension boundaries

The runtime recognizes these extension descriptor boundaries:

- `TaskAdapter`
- `ExecutorBackend`
- `ArtifactStore`
- `ObservabilitySink`

Each descriptor must declare:

- `plugin_name`
- `plugin_version`
- `boundary`
- `contract_version`
- `capabilities`
- `trust_model`

The JSON schema in `configs/dag/schema/extension_descriptor.schema.json` is the
authoritative machine-readable descriptor shape.

## Stability and ownership

The governed extension points are:

- `task_adapter`: stable, owned by runtime
- `executor_backend`: experimental, owned by runtime
- `validation_hook`: internal hook, owned by core

Internal hooks are not public extension points until they satisfy the
promotion checklist in
`docs/spec/INTERNAL_HOOK_PROMOTION_CHECKLIST.md`.

## Compatibility and failure isolation

- `contract_version` must use the `v`-prefixed format
- registration conflicts must be rejected by plugin name
- compatibility inspection must report unsupported contract versions and
  missing required capabilities
- extension failures must remain isolated to the extension boundary and must
  not be treated as acceptable if the engine crashes

## Related tests

- `crates/bijux-dag-runtime/tests/extension_catalog_contracts.rs`

## Versioning and change policy

Any incompatible change to extension boundary kinds, descriptor requirements,
stability classification, compatibility semantics, or failure-isolation rules
must update this contract, the descriptor schema, and the linked tests in the
same change.
