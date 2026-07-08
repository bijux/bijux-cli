---
title: Data Contracts
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
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

Use [Graph Schema Reference](reference/graph-schema.md) for the full DAG
file-format reference, including top-level graph fields, reusable subgraphs,
dynamic controllers, trigger rules, path variables, and validation diagnostics.

- `graph.inputs` is a JSON object of graph-level input values available during
  planning and execution.
- `node.inputs` declares named input ports. Edges bind those ports to upstream
  output names.
- `node.outputs` declares typed output contracts with a stable `name`, relative
  `path`, optional `kind`, optional `required`, optional `media_type`, and
  optional `promotable`.
- `node.params` accepts literal JSON, arrays, objects, and reference objects.
- `timeout_ms`, `resources`, `retry`, `effects`, `env_allowlist`, `cache`,
  `branch`, and `dynamic` are part of the live graph shape.

### Output Contract Shape

Output kinds are explicit rather than inferred from file names:

- `file`
- `directory`
- `value`
- `table`
- `log`
- `binary`
- `bundle`

The minimal output declaration is still:

```json
{ "name": "report", "path": "report.json" }
```

That defaults to a required `file` output with media type
`application/octet-stream`.

A richer declaration can tighten the contract:

```json
{
  "name": "primary",
  "path": "primary.json",
  "kind": "value",
  "media_type": "application/json"
}
```

Optional outputs stay declared and observable instead of silently disappearing:

```json
{
  "name": "secondary",
  "path": "secondary.txt",
  "kind": "log",
  "required": false,
  "media_type": "text/plain"
}
```

Run traces record whether each declared output was present, along with the
resolved media type and sha256 digest for materialized outputs.

## Input Materialization Contract

Downstream node inputs are materialized into a stable per-node directory:

- `nodes/<node_id>/inputs/<source_node_id>/<input_port>`
- `nodes/<node_id>/inputs/index.json`

The index is execution evidence, not an incidental helper file. Each entry
records:

- `local_path`: the materialized path relative to the node input root
- `source_node_id`: the upstream node that produced the input
- `source_node_fingerprint`: the resolved upstream node fingerprint
- `source_output_name`: the upstream output contract name
- `source_sha256`: the upstream artifact digest
- `materialization_mode`: `copy`, `hardlink`, or `symlink`

The runtime verifies that the materialized input matches the recorded upstream
digest before execution proceeds. That keeps the input index honest across all
supported materialization modes and makes downstream cache identity depend on
the actual upstream content that was wired into the node.

### Reference Shapes

Use the graph schema reference for the complete authoring contract. The
execution-facing summary here keeps the shapes close to how downstream input
materialization reasons about them.

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

Graph params can also bind to stable node-local execution directories through
graph-level path variables:

```json
{ "path_var": "outputs_dir" }
```

```json
{
  "path_var": {
    "name": "cache_dir",
    "relative_path": "aligned/sample-a.bam"
  }
}
```

The path-variable contract is intentionally narrow:

- supported variables are `run_dir`, `work_dir`, `inputs_dir`, `outputs_dir`,
  and `cache_dir`
- `relative_path` stays normalized and relative; traversal such as `../` is
  rejected during validation
- host-node params may also use whole-string brace expressions such as
  `"{outputs_dir}/result.txt"` inside JSON scalar values
- container `argv` and `workdir` accept the same whole-string brace
  expressions, but container-visible bindings resolve to the container mount
  paths rather than the host staging paths
- literal absolute container `workdir` values are governed by the runtime
  absolute-path policy instead of being guessed or rewritten

### Execution Policy Fields

- `resources` supports `cpu` and `mem_mb`, plus optional `gpu_devices` and
  `named_resources`.
- `retry` currently supports `max_attempts` and `backoff_ms`.
- `timeout_ms` is a first-class node field rather than an adapter-specific
  parameter convention.
- `cache.enabled = false` requires a non-empty `cache.reason` so cache opt-out is
  auditable.
- `env_allowlist` is only valid when the node declares the `env` effect.
- `container.env_allowlist`, `container.workdir`, and container `argv` all stay
  inside the authored graph contract rather than being inferred later.
- declared outputs, env rules, params, and cache policy all affect operator and
  release-facing contract surfaces.

### Validation Guarantees

Strict parsing and validation reject malformed graph contracts instead of
guessing:

- unknown fields are rejected
- malformed references are rejected
- a reference must specify exactly one of `graph_input`, `node_output`, or `path_var`
- cache disablement without a reason is rejected
- references to undeclared graph inputs or missing node outputs are rejected
- unknown path variables and traversal-bearing path suffixes are rejected
- unsupported container engines are rejected
- invalid reusable-subgraph bindings and branch decisions are rejected

## Plan Preview Contract

`plan explain`, `show-effective-plan`, and JSON scheduling payloads emitted by
`run --preflight-only` or `run --explain-scheduling` can include a resolved path
preview when the caller supplies a run root.

The preview payload exposes:

- `run_layout`: the previewed `run_id`, staging path, and final path
- `absolute_path_policy`: the policy used to judge literal absolute container
  workdirs
- `path_previews`: per-node bindings plus the resolved path expressions that
  were found in params, container `argv`, or container `workdir`

The planner preview is advisory rather than proof of successful execution, but
it uses the same run-id selection and path-binding rules that the runtime uses
when the run is actually launched.

## Run Manifest Input Contract

When a workflow is launched with runtime input overrides, the run artifact
contract records the effective inputs in `manifest.json` under
`run_metadata.graph_inputs`.

Graph inputs may use shorthand defaults such as `"region": "eu-west-1"` or an
explicit typed schema such as
`"region": { "type": "string", "default": "eu-west-1" }`.

- the recorded values reflect the merged runtime view that executed
- `--inputs-file` values can be overridden by later `--input key=value` flags
- operator-facing human output redacts secret-like keys, but the manifest
  preserves the effective input values for replay and audit context

## Schedule Trigger Input Contract

Schedule definitions can bind trigger-derived context into typed graph inputs
before a submission request is issued.

- `input_contract` declares the allowed graph inputs and their types
- `input_bindings` maps each declared input to a trigger source
- dependency triggers declare `trigger.Dependency.on_status` as
  `success`, `failure`, or `any_terminal`
- bound values are normalized with the same graph-input materialization rules
  used by direct runtime inputs
- a schedule that cannot produce a required bound input is rejected before
  submission instead of issuing a partially formed run request

The supported binding sources are:

- `requested_unix_ms`
- `manual_argument`
- `event_payload`
- `signal_payload`
- `dependency_upstream_run_id`
- `dependency_status`
- `backfill_window_start_unix_ms`
- `backfill_window_end_unix_ms`
- `backfill_partition_key`

Payload bindings may target either the whole payload or a JSON Pointer inside
the payload. Pointer values must be empty or begin with `/`.

Event-triggered submissions also retain `event_lineage` with the originating
`event_id`, `event_type`, `source`, and `occurred_unix_ms` in both the
generated submission request and the durable submission ledger.

Dependency trigger conditions use the following terminal outcome classes:

- `success`: successful upstream completions
- `failure`: terminal non-success completions such as failed, cancelled, or
  timed-out runs
- `any_terminal`: either successful or failure terminal outcomes

### Example

```json
{
  "id": "event-ingest",
  "dag_name": "atlas.event-ingest",
  "dag_version_policy": "run-latest",
  "input_contract": {
    "event_tenant": { "type": "string", "required": true },
    "event_payload": { "type": "object", "required": true }
  },
  "input_bindings": {
    "event_tenant": {
      "source": "event_payload",
      "pointer": "/tenant"
    },
    "event_payload": {
      "source": "event_payload"
    }
  },
  "trigger": {
    "Event": {
      "event_type": "dataset.ready",
      "source": "catalog"
    }
  }
}
```

When the scheduler evaluates this definition against an event payload such as
`{"tenant":"atlas","batch":7}`, the generated submission request records typed
graph inputs for both `event_tenant` and `event_payload`.

### Example

`evidence/dag/authoring/examples/parameterized-report.dag.json` is the
repository’s reference example for this contract shape. Its `publish_summary`
node combines graph inputs, a downstream output reference, explicit retry and
timeout policy, environment allowlisting, and a justified cache opt-out:

```json
{
  "inputs": {
    "dataset_uri": {
      "type": "string",
      "default": "s3://warehouse/catalog"
    },
    "publish_channel": {
      "type": "enum",
      "values": ["daily-summary", "weekly-summary"],
      "default": "daily-summary"
    },
    "region": {
      "type": "string",
      "default": "eu-west-1"
    }
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
