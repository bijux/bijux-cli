---
title: Data Contracts
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-04
---

# Data Contracts

This page explains the data shapes that let DAG definitions, runs, artifacts,
and comparisons stay inspectable across tools and time.

The important split is not model count. It is whether a contract describes the
graph itself, execution evidence, artifact evidence, or comparison outcomes.

## Contract Map

```mermaid
flowchart LR
    contracts["dag contracts"] --> graph["graph and validation models"]
    contracts --> plans["plan and execution models"]
    contracts --> state["run and trace models"]
    contracts --> artifacts["artifact and lineage models"]
    contracts --> compare["replay and diff outcomes"]
```

## Contract Families

- graph model and validation diagnostics
- execution plan and scheduler state representations
- node outcomes, run summaries, and timeline events
- artifact metadata, integrity proofs, and lineage links
- replay/diff classification payloads and reason codes

## Graph Authoring Contract

The graph contract in `bijux-dag/v0.1` is intentionally explicit. A serious
graph file can declare graph-scoped inputs, node-local params, output
contracts, execution policy, and environment policy without relying on
undocumented adapter behavior.

- `graph.inputs` is a JSON object of graph-level input values available during
  planning and execution.
- `node.inputs` declares named input ports. Edges bind those ports to upstream
  output names.
- `node.outputs` declares output contracts as `{ "name": ..., "path": ... }`.
- `node.params` accepts literal JSON, arrays, objects, and reference objects.
- `timeout_ms`, `resources`, `retry`, `effects`, `env_allowlist`, and `cache`
  are part of the stable graph shape.

### Reference Shapes

Graph params can bind to graph inputs:

```json
{ "graph_input": "region" }
```

Graph params can also bind to a downstream-consumable output from another node:

```json
{
  "node_output": {
    "node_id": "build_report",
    "output_name": "report"
  }
}
```

The legacy `path` field is still accepted as an alias inside `node_output`
references for compatibility, but durable docs and examples use `output_name`.

### Execution Policy Fields

- `resources` currently supports `cpu` and `mem_mb`.
- `retry` currently supports `max_attempts` and `backoff_ms`.
- `cache.enabled = false` requires a non-empty `cache.reason` so cache opt-out is
  auditable.
- `env_allowlist` is only valid when the node declares the `env` effect.
- declared outputs, env rules, params, and cache policy all affect operator and
  release-facing contract surfaces.

### Validation Guarantees

Strict parsing and validation reject malformed graph contracts instead of
guessing:

- unknown fields are rejected
- malformed references are rejected
- a reference must specify exactly one of `graph_input` or `node_output`
- cache disablement without a reason is rejected
- references to undeclared graph inputs or missing node outputs are rejected

### Example

`evidence/dag/authoring/examples/parameterized-report.dag.json` is the
repository’s reference example for this contract shape. Its `publish_summary`
node combines graph inputs, a downstream output reference, explicit retry and
timeout policy, environment allowlisting, and a justified cache opt-out:

```json
{
  "inputs": {
    "dataset_uri": "s3://warehouse/catalog",
    "publish_channel": "daily-summary",
    "region": "eu-west-1"
  },
  "nodes": [
    {
      "id": "publish_summary",
      "params": {
        "channel": { "graph_input": "publish_channel" },
        "report_source": {
          "node_output": {
            "node_id": "build_report",
            "output_name": "report"
          }
        }
      },
      "cache": {
        "enabled": false,
        "reason": "publishes externally visible summary"
      },
      "effects": ["filesystem", "env"],
      "env_allowlist": ["REPORT_CHANNEL"]
    }
  ]
}
```

## Code Anchors

- `crates/bijux-dag-core/src/graph/model.rs`
- `crates/bijux-dag-core/src/graph/node.rs`
- `crates/bijux-dag-core/src/contracts/error.rs`
- `crates/bijux-dag-core/src/pipeline/validate.rs`
- `crates/bijux-dag-runtime/src/runtime_core/`
- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-app/src/routes/response.rs`

## Contract Rules

- contract-bearing fields should stay explicit and test-covered
- identity-related field semantics require compatibility review
- classification states must remain machine-parseable

## Reading Rule

Use this page when a DAG change crosses graph, run, artifact, or diff
boundaries and the hard part is deciding which observable contract is at stake.

## Next Reads

- [Artifact Contracts](artifact-contracts.md)
- [Compatibility Commitments](compatibility-commitments.md)
- [Invariants](../quality/invariants.md)
