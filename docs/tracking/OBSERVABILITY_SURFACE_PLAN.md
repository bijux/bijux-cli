# Observability Surface Plan

## Operator Surface

- stable event names
- stable metric names
- concise run inspect output
- failure summary and root causes

## Developer Surface

- full timeline entries
- debug-only correlation IDs
- verbose adapter/backend internals

## Boundary Policy

- operator surface is contract-governed and backward-compatible.
- developer surface may evolve with debug tooling and is not a stability promise.
