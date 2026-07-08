---
title: Graph Schema Reference
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Graph Schema Reference

This page is the field-by-field reference for `bijux-dag/v0.1` graph files.
Use it when you are authoring a workflow, reviewing a pull request, or wiring
editor tooling to the checked-in schema.

## Source Of Truth

Two repository surfaces define the graph contract together:

- strict parsing and validation in `crates/bijux-dag-core/src/graph/`
- the editor-facing JSON Schema at `configs/dag/schema/dag.schema.json`

The parser and validator are authoritative when a question is about accepted
input, canonicalization, or validation behavior. The JSON Schema is the right
entry point for editors, completion, and repository-local schema validation.

## Top-Level Graph Shape

A graph file is a JSON object with three required fields:

- `spec`
- `nodes`
- `edges`

The supported schema version today is `bijux-dag/v0.1`.

```json
{
  "spec": "bijux-dag/v0.1",
  "meta": {
    "name": "parameterized-report",
    "description": "Build and publish one report",
    "owners": ["platform"],
    "tags": ["reporting"]
  },
  "inputs": {
    "dataset_uri": {
      "type": "string",
      "default": "s3://warehouse/catalog"
    }
  },
  "nodes": [],
  "edges": []
}
```

Top-level fields:

- `spec`: the graph schema version. `bijux-dag/v0.1` is the only accepted
  value today.
- `meta`: optional human-facing metadata. `name` is required when `meta` is
  present. `description`, `owners`, and `tags` are optional.
- `inputs`: optional graph-scoped input declarations.
- `nondeterminism_allowed`: optional boolean for graphs that intentionally rely
  on non-seeded nondeterminism.
- `subgraphs`: optional reusable DAG fragments defined inside the same file.
- `subgraph_instances`: optional reusable-fragment instantiations.
- `nodes`: the node list.
- `edges`: the edge list.

## Graph Inputs

`graph.inputs` declares runtime-provided or defaulted values that nodes can
consume through references.

Shorthand form uses a default value directly:

```json
{
  "inputs": {
    "region": "eu-west-1",
    "retry_limit": 2
  }
}
```

Explicit form declares type and policy:

```json
{
  "inputs": {
    "source_note": {
      "type": "path",
      "required": true
    },
    "publish_channel": {
      "type": "enum",
      "values": ["daily-summary", "weekly-summary"],
      "default": "daily-summary"
    }
  }
}
```

Supported graph input kinds:

- `string`
- `integer`
- `float`
- `boolean`
- `path`
- `enum`
- `array`
- `object`

Input schema fields:

- `type`: required in the explicit form.
- `required`: optional boolean. When omitted, the input is optional unless the
  caller provides an explicit policy elsewhere.
- `default`: optional default value recorded into the effective input contract.
- `values`: required for `enum`.
- `items`: required for `array` and declares the item kind.
- `properties`: optional for `object` and declares named child fields.

`array.items` accepts only the item-type surface, not a second full
`required/default` envelope.

## Node Fields

Each entry in `graph.nodes` is a workflow node. Only `id` and `kind` are
required by the parser; the remaining fields default when omitted.

- `id`: stable node identity used by edges, references, run directories, and
  traces.
- `kind`: node adapter kind such as `const`, `shell`, `python`, `http`,
  `file_transform`, or `container`. Other strings are preserved for external
  adapter families.
- `semantic_kind`: optional execution meaning. Current values are `task`,
  `branch`, `barrier`, `map`, `reduce`, and `dynamic`.
- `inputs`: declared input port names.
- `outputs`: declared output contracts.
- `params`: literal JSON, nested arrays and objects, or reference objects.
- `container`: container execution contract for `container` nodes.
- `timeout_ms`: optional per-node timeout in milliseconds.
- `resources`: optional scheduling resource request.
- `tags`: optional labels that stay attached to the node contract.
- `retry`: optional retry policy. Defaults to no retries.
- `cache`: optional cache behavior. Defaults to enabled.
- `effects`: declared side effects such as filesystem or environment access.
- `env_allowlist`: exact environment variables the node may read when the node
  declares the `env` effect.
- `group`: optional logical node grouping.
- `trigger_rule`: optional upstream completion rule for scheduling.
- `branch`: branch contract for `semantic_kind: "branch"` nodes.
- `dynamic`: dynamic expansion contract for `semantic_kind: "dynamic"` nodes.

## Params And Reference Objects

`node.params` accepts four shapes:

- primitive JSON values
- arrays
- objects
- one reference object

Reference objects come in exactly three families.

Graph input reference:

```json
{ "graph_input": "publish_channel" }
```

Upstream output reference:

```json
{
  "node_output": {
    "node_id": "build_report",
    "output_name": "report"
  }
}
```

Path variable reference:

```json
{ "path_var": "outputs_dir" }
```

Path variables can also carry a relative suffix:

```json
{
  "path_var": {
    "name": "cache_dir",
    "relative_path": "aligned/sample-a.bam"
  }
}
```

Compatibility note:

- `node_output.path` is still accepted as an alias for `node_output.output_name`
  when reading legacy JSON
- durable docs and new examples should use `output_name`

## Output Contracts

`node.outputs` declares the files, directories, or value payloads a node is
expected to materialize.

Minimal output declaration:

```json
{
  "name": "report",
  "path": "report/report.json"
}
```

Richer output declaration:

```json
{
  "name": "bulletin",
  "path": "publish/bulletin.md",
  "kind": "file",
  "required": true,
  "media_type": "text/markdown",
  "promotable": true
}
```

Output fields:

- `name`: stable output contract name.
- `path`: relative output path below the node output root.
- `kind`: optional output kind. Defaults to `file`.
- `required`: optional boolean. Defaults to `true`.
- `media_type`: optional explicit media type. When omitted, the runtime uses
  the default media type for the chosen output kind.
- `promotable`: optional boolean. Use it for outputs intended to move into a
  retained publication or artifact lane.

Current output kinds:

- `file`
- `directory`
- `value`
- `table`
- `log`
- `binary`
- `bundle`

## Path Variables

The graph contract exposes five built-in path variables:

- `run_dir`
- `work_dir`
- `inputs_dir`
- `outputs_dir`
- `cache_dir`

They can appear in reference objects and in whole-string brace expressions
inside params, container `argv`, or container `workdir`.

Examples:

```json
{ "path_var": "outputs_dir" }
```

```json
{
  "container": {
    "workdir": "{work_dir}/scratch",
    "argv": ["/bin/sh", "-c", "ls {inputs_dir}"]
  }
}
```

Path-variable suffixes must stay normalized and relative. Traversal like
`../escape` is rejected during validation.

## Code Anchors

- `configs/dag/schema/dag.schema.json`
- `crates/bijux-dag-core/src/graph/input.rs`
- `crates/bijux-dag-core/src/graph/model.rs`
- `crates/bijux-dag-core/src/graph/node.rs`

## Reading Rule

Use this page when the question is about which fields a DAG file may declare.
Move to [Reusable Subgraphs](../guides/reusable-subgraphs.md) when the next
question is about graph reuse, or back to [Data Contracts](../data-contracts.md)
when the next question is about run manifests, artifacts, or replay payloads.
