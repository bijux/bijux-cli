# Placeholder Inventory Report

## Summary

This inventory captures placeholder modules, commands, functions, and output surfaces that still exist.

## Classification

| Surface | Classification | Owner | Decision |
| --- | --- | --- | --- |
| `crates/bijux-dag-cli/src/main.rs` reserved `rag` command | command surface | dag-cli | postpone-with-doc-only |
| `crates/bijux-dag-cli/src/main.rs` reserved `rar` command | command surface | dag-cli | postpone-with-doc-only |
| `crates/bijux-dag-artifacts/src/io/store.rs` object store runtime boundary | function/output surface | dag-artifacts | postpone-with-doc-only |
| `configs/schema/fixtures/v0.2-draft/positive/placeholder.json` | metadata fixture | schema-governance | delete-now |

## Delete-Now

- `configs/schema/fixtures/v0.2-draft/positive/placeholder.json` (replaced by `minimal_empty_graph.json`).

## Implement-Now

- None in this pass.

## Postpone-With-Doc-Only

- `cli-reserved-rag-rar`
- `runtime-object-store-boundary`
