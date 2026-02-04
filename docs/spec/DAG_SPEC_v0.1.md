# DAG Spec v0.1

## Overview
A DAG is a JSON document with `spec`, `nodes`, and `edges`. Nodes define typed operations and named ports. Edges connect output ports to input ports.

## Graph
```
{
  "spec": "bijux-dag/v0.1",
  "nodes": [Node],
  "edges": [Edge]
}
```

## Node
```
{
  "id": "string",
  "kind": "const|shell|container",
  "inputs": ["string"],
  "outputs": ["string"],
  "params": <ParamValue>,
  "container": <ContainerSpec>,
  "effects": ["filesystem|network|env"],
  "env_allowlist": ["ENV_VAR"],
  "container": <ContainerSpec>
}
```
- `id` must be unique and match `[a-zA-Z0-9_-]+`.
- `kind` determines executor behavior.
- `inputs` and `outputs` list valid ports.
- `params` is executor-specific.
- `effects` is required for `shell` nodes.
- `env_allowlist` lists env vars allowed for `shell` nodes.
- `container` is required for `container` nodes.

### const params
```
{"value": <json>}
```

### shell params
```
{"argv": ["cmd", "arg1", ...]}
```

## ContainerSpec
```
{
  "image": "string",
  "command": ["string"],
  "args": ["string"],
  "env_allowlist": ["ENV_VAR"],
  "mounts": [{"source": "inputs/...", "target": "/bijux/node/...", "read_only": bool}],
  "workdir": "/bijux/node/work"
}
```

## ParamValue
Params can be literal JSON values or explicit references. No string templating is allowed in v0.1.

```
<ParamValue> =
  <literal json>
  | {"graph_input": "name"}
  | {"node_output": {"node_id": "id", "path": "output_port"}}
  | {"key": <ParamValue>, ...}
  | [<ParamValue>, ...]
```

## Edge
```
{
  "from": {"node_id": "string", "port": "string"},
  "to": {"node_id": "string", "port": "string"}
}
```

## PortRef
```
{"node_id": "string", "port": "string"}
```

## Strictness
- Unknown fields are rejected.
- Missing required fields are rejected.

## Canonicalization
- Nodes are sorted by `id`.
- Edges are sorted by (`from.node_id`, `from.port`, `to.node_id`, `to.port`).
- `inputs` and `outputs` are sorted.
- `params` object keys are sorted recursively.
