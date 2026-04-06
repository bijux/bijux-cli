---
title: Extensibility Model
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Extensibility Model

`bijux-cli` extensibility is route-centric. Extensions are integrated through
plugin manifests, namespace registration, compatibility checks, and entrypoint
execution contracts.

## Visual Summary

```mermaid
flowchart LR
    manifest["plugin.manifest.json"] --> validate["manifest and compatibility validation"]
    validate --> register["namespace registration"]
    register --> route["route resolution"]
    route --> execute["python delegated or external-exec runtime"]
    execute --> report["structured report or process result"]
```

## Extensibility Layers

- manifest schema and field validation
- reserved namespace and alias conflict protection
- compatibility range checks against host semver
- lifecycle toggles (`enable`, `disable`, `uninstall`)
- diagnostics surfaces (`doctor`, `explain`, `schema`, `where`)

## Code Anchors

- `crates/bijux-cli/src/contracts/plugin.rs`
- `crates/bijux-cli/src/features/plugins/manifest.rs`
- `crates/bijux-cli/src/features/plugins/operations.rs`
- `crates/bijux-cli/src/features/plugins/runtime.rs`
- `crates/bijux-cli/src/routing/registry.rs`

## Extensibility Constraints

- no full sandbox isolation guarantee
- extension namespaces must not collide with reserved roots
- compatibility declarations must be semver-parseable
- entrypoint execution errors must remain inspectable

## Next Reads

- [Artifact Contracts](../interfaces/artifact-contracts.md)
- [Security and Safety](../operations/security-and-safety.md)
- [Known Limitations](../quality/known-limitations.md)
