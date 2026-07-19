---
title: Reusable Subgraphs
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
---

# Reusable Subgraphs

Reusable subgraphs let operators define one workflow fragment once and
instantiate it multiple times without duplicating the internal node list.

The contract is authoring-oriented on input, but plain-DAG-oriented after
expansion:

- authors define reusable blocks under `graph.subgraphs`
- authors instantiate them through `graph.subgraph_instances`
- compilation, validation, planning, and graph identity all operate on the
  expanded plain graph

## Authoring Shape

A reusable block has two parts:

- `graph`: the nested DAG fragment, including its own `inputs`, `nodes`, and `edges`
- `outputs`: the named outputs that the parent graph may reference

An instance declares:

- `id`: the stable namespace prefix for the expanded nodes
- `subgraph`: the reusable block to instantiate
- `input_bindings`: bindings for the reusable block inputs

```json
{
  "subgraphs": {
    "align_block": {
      "graph": {
        "spec": "bijux-dag/v0.1",
        "inputs": {
          "sample_name": { "type": "string" }
        },
        "nodes": [
          {
            "id": "extract",
            "kind": "const",
            "outputs": [{ "name": "sheet", "path": "extract/sheet.txt" }],
            "params": {
              "sample": { "graph_input": "sample_name" }
            }
          },
          {
            "id": "align",
            "kind": "const",
            "inputs": ["sheet"],
            "outputs": [{ "name": "bam", "path": "align/result.bam" }]
          }
        ],
        "edges": [
          {
            "from": { "node_id": "extract", "port": "sheet" },
            "to": { "node_id": "align", "port": "sheet" }
          }
        ]
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

## Binding Rules

- reusable-block inputs are bound through `subgraph_instances[].input_bindings`
- a binding may be a literal, a `graph_input` reference, or a `node_output`
  reference
- if a reusable-block input has a default in the nested `graph.inputs`, the
  instance may omit that binding
- edges may read from an exposed reusable-block output, but edges may not target
  reusable-block inputs directly

Input ports stay explicit at the reusable-block boundary, but the parent graph
connects them through bindings instead of by drawing edges into the instance id.

## Expansion Rules

Expansion is deterministic:

- each nested node id becomes `<instance_id>__<local_node_id>`
- each nested output path becomes `subgraphs/<instance_id>/<nested_output_path>`
- exposed reusable-block outputs are rewritten to the concrete expanded producer
- nested reusable blocks expand before the parent block exports are resolved

For the example above, `tumor_align.align` becomes the plain node
`tumor_align__align`.

## Identity And Validation

- graph fingerprints and graph ids are computed from the expanded graph
- validation runs against the expanded graph plus reusable-block boundary rules
- reusable-block instance order does not affect graph identity when the expanded
  graph is otherwise equivalent
- invalid exports, missing bindings, duplicate instance ids, and edges into
  reusable-block inputs are rejected before planning or execution

## Code Anchors

- `crates/bijux-dag-core/src/graph/model.rs`
- `crates/bijux-dag-core/src/graph/expansion.rs`
- `crates/bijux-dag-core/src/build/compile.rs`
- `crates/bijux-dag-core/tests/subgraph_expansion_contract.rs`

## Reading Rule

Use this page when the question is about how to author or consume reusable DAG
fragments. Move to Data Contracts when the next question is about generic
reference shapes, graph inputs, or run artifact payloads.
