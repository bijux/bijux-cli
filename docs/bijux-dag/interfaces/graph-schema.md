---
title: Graph Schema Reference
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Graph Schema Reference

This is the field reference for `bijux-dag/v0.1` graph files. It explains
accepted authoring shape; it does not duplicate runtime evidence or operator
procedures.

## Contract Authority

| Question | Authority |
| --- | --- |
| editor completion and structural JSON validation | `configs/dag/schema/dag.schema.json` |
| accepted input and canonical decoding | `crates/bijux-dag-core/src/graph/model.rs` |
| topology, references, paths, and semantic validation | `crates/bijux-dag-core/src/pipeline/validate.rs` |
| authoring workflow and rejection examples | [Authoring Guide](authoring-guide.md) |

The Rust parser and validator decide whether a graph is accepted. JSON Schema
cannot express every cross-reference, topology, branch, or path rule.

Start from a checked-in graph rather than assembling isolated snippets:

| Need | Graph |
| --- | --- |
| minimal execution | `evidence/dag/authoring/examples/hello.dag.json` |
| typed inputs and report output | `evidence/dag/authoring/examples/file-processing-report.dag.json` |
| conditional lanes | `evidence/dag/authoring/examples/audience-branch-bulletin.dag.json` |
| container execution | `evidence/dag/authoring/examples/release-note-bundle.dag.json` |

```bash
bijux-dag validate ./workflow.dag.json
bijux-dag plan explain ./workflow.dag.json --json
```

## Top-Level Shape

`spec`, `nodes`, and `edges` are required:

```json
{
  "spec": "bijux-dag/v0.1",
  "meta": {
    "name": "parameterized-report",
    "owners": ["platform"],
    "tags": ["reporting"]
  },
  "inputs": {
    "region": {
      "type": "string",
      "default": "eu-west-1"
    }
  },
  "nodes": [],
  "edges": []
}
```

| Field | Meaning |
| --- | --- |
| `spec` | schema version; `bijux-dag/v0.1` is accepted |
| `meta` | optional name, description, owners, and tags |
| `inputs` | optional graph-scoped input declarations |
| `nondeterminism_allowed` | explicit opt-in for non-seeded nondeterminism |
| `subgraphs` | reusable graph fragments declared in the file |
| `subgraph_instances` | instantiations of reusable fragments |
| `nodes` | node contracts |
| `edges` | dependency and conditional-routing contracts |

## Graph Inputs

`graph.inputs` supports shorthand defaults:

```json
{
  "inputs": {
    "region": "eu-west-1",
    "retry_limit": 2
  }
}
```

Use explicit declarations when type or requiredness matters:

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

Supported kinds are `string`, `integer`, `float`, `boolean`, `path`, `enum`,
`array`, and `object`.

| Field | Rule |
| --- | --- |
| `type` | required for explicit declarations |
| `required` | marks a value that must resolve before execution |
| `default` | becomes part of the effective input contract |
| `values` | required for `enum` |
| `items` | required for `array`; declares item kind |
| `properties` | declares named object children |

## Node Fields

Only `id` and `kind` are required by the parser.

| Field | Meaning |
| --- | --- |
| `id` | stable identity used by edges, references, traces, and run paths |
| `kind` | adapter kind such as `const`, `shell`, `python`, `http`, `file_transform`, or `container` |
| `semantic_kind` | `task`, `branch`, `barrier`, `map`, `reduce`, or `dynamic` |
| `inputs` | declared input-port names |
| `outputs` | declared output contracts |
| `params` | literal JSON or reference objects |
| `container` | image, argv, environment, workdir, and engine contract |
| `timeout_ms` | optional node timeout |
| `resources` | scheduler-visible resource request |
| `retry` | attempt and backoff policy |
| `cache` | cache eligibility and opt-out reason |
| `effects` | declared filesystem, network, environment, or clock effects |
| `env_allowlist` | exact environment names available with the `env` effect |
| `tags`, `group` | routing and logical grouping metadata |
| `trigger_rule` | upstream completion rule |
| `branch` | decisions and decision-output contract |
| `dynamic` | runtime expansion-output contract |

## Params And References

Params may contain ordinary JSON or one of three reference families:
`graph_input`, `node_output`, or `path_var`.

Graph input:

```json
{ "graph_input": "publish_channel" }
```

Upstream output:

```json
{
  "node_output": {
    "node_id": "build_report",
    "output_name": "report"
  }
}
```

Runtime path:

```json
{
  "path_var": {
    "name": "cache_dir",
    "relative_path": "aligned/sample-a.bam"
  }
}
```

`node_output.path` is accepted as a compatibility alias for
`node_output.output_name`; new graphs use `output_name`.

## Output Contracts

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

| Field | Rule |
| --- | --- |
| `name` | stable output identity |
| `path` | normalized relative path below the node output root |
| `kind` | `file`, `directory`, `value`, `table`, `log`, `binary`, or `bundle`; defaults to `file` |
| `required` | defaults to `true` |
| `media_type` | explicit interpretation; otherwise kind default |
| `promotable` | marks output eligible for governed promotion |

Output declaration does not prove materialization. Runtime evidence records
presence and digest.

## Execution Policy

| Field | Contract |
| --- | --- |
| `timeout_ms` | node deadline in milliseconds |
| `resources.cpu`, `resources.mem_mb` | required when `resources` is present |
| `resources.gpu_devices` | optional accelerator count |
| `resources.named_resources` | named capacities such as licenses or database slots |
| `retry.max_attempts` | total attempts including the first |
| `retry.backoff_ms` | fixed delay before another attempt |
| `cache.enabled` | defaults to enabled |
| `cache.reason` | required and non-empty when cache is disabled |
| `effects` | declared `filesystem`, `network`, `env`, and `clock` use |
| `env_allowlist` | valid only with the `env` effect |

Example:

```json
{
  "timeout_ms": 30000,
  "resources": {
    "cpu": 2,
    "mem_mb": 1024,
    "gpu_devices": 0,
    "named_resources": {
      "database_slot": 1
    }
  },
  "retry": {
    "max_attempts": 2,
    "backoff_ms": 1000
  },
  "cache": {
    "enabled": false,
    "reason": "publishes an external side effect"
  },
  "effects": ["filesystem", "env"],
  "env_allowlist": ["REPORT_CHANNEL"]
}
```

## Container Contract

```json
{
  "container": {
    "image": "docker.io/library/alpine:3.20@sha256:...",
    "argv": ["/bin/sh", "-c", "cp /bijux/node/inputs/in /bijux/node/outputs/out"],
    "env_allowlist": ["REPORT_CHANNEL"],
    "workdir": "{work_dir}/scratch",
    "engine": "docker"
  }
}
```

`image` and `argv` define execution. `env_allowlist` defines visible
environment names. `workdir` may use path variables. `engine` is `docker` or
`podman`. See [Execution Security And Isolation](../operations/security-isolation-truth.md)
before treating this contract as a host-security boundary.

## Branches And Triggers

A branch node declares `semantic_kind: "branch"`, allowed `decisions`, and a
`decision_output`. Conditional edges bind a decision to a downstream lane.

Trigger rules are `all_success`, `any_success`, `all_done`, and `none_failed`.
Validation rejects a conditional decision absent from the owning branch.

## Reusable And Dynamic Graphs

`subgraphs` declares an embedded fragment; `subgraph_instances` binds and
instantiates it:

```json
{
  "subgraphs": {
    "align_block": {
      "graph": {
        "spec": "bijux-dag/v0.1",
        "nodes": [],
        "edges": []
      },
      "outputs": {
        "aligned": {
          "node_id": "align",
          "output_name": "bam"
        }
      }
    }
  },
  "subgraph_instances": [
    {
      "id": "tumor_align",
      "subgraph": "align_block",
      "input_bindings": {
        "sample_name": { "graph_input": "sample" }
      }
    }
  ]
}
```

Compilation expands reusable fragments before planning and identity decisions.
[Reusable Subgraphs](reusable-subgraphs.md) owns the complete binding contract.

A node with `semantic_kind: "dynamic"` may declare:

```json
{
  "dynamic": {
    "expansion_output": "expansion"
  }
}
```

Dynamic expansion is an advanced controller surface and participates in graph
identity and validation.

## Path Variables

Built-in names are `run_dir`, `work_dir`, `inputs_dir`, `outputs_dir`, and
`cache_dir`. They may appear as references or whole-string expressions in
params, container argv, and container workdir.

Suffixes must be normalized and relative. Traversal such as `../escape` is
rejected.

## Validation Diagnostics

Strict parsing rejects unknown fields, malformed references, and wrong JSON
shapes. Semantic validation then resolves references, topology, paths,
branches, effects, engines, and cache policy.

Validation diagnostics always carry:

- `code`
- `message`
- `path`
- `hint`
- `severity`

Rejected conditions include undeclared inputs, missing outputs, path escapes,
invalid branch decisions, `env_allowlist` without `env`, unsupported container
engines, and cache opt-out without a reason.

## Code Anchors

- `configs/dag/schema/dag.schema.json`
- `crates/bijux-dag-core/src/graph/model.rs`
- `crates/bijux-dag-core/src/graph/node.rs`
- `crates/bijux-dag-core/src/graph/input.rs`
- `crates/bijux-dag-core/src/pipeline/validate.rs`

## Related Authorities

- [Authoring Guide](authoring-guide.md)
- [Reusable Subgraphs](reusable-subgraphs.md)
- [Data Contracts](data-contracts.md)
- [Error Codes](error-codes.md)
