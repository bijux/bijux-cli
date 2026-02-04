# Run Artifact Spec v0.1

## Run Directory Layout
```
run-<id>/
  manifest.json
  provenance.json
  graph.snapshot.json
  outputs/
    index.json
  nodes/
    <node_id>/
      trace.json
      resolved_params.json
      stdout.log
      stderr.log
      inputs/
        index.json
        <files>
      outputs/
        index.json
        <files>
```

## manifest.json
```
{
  "run_id": "string",
  "created_unix_ms": number,
  "graph_snapshot": "graph.snapshot.json",
  "status": "success|failed|cancelled",
  "spec": "bijux-dag/v0.1"
}
```

## graph.snapshot.json
```
{
  "graph": <canonical graph>,
  "graph_fingerprint": "sha256"
}
```

## trace.json
```
{
  "node_id": "string",
  "status": "success|failed|skipped|cached",
  "started_unix_ms": number,
  "finished_unix_ms": number,
  "fingerprint": "sha256"
}
```

## provenance.json
```
{
  "os": "string",
  "arch": "string",
  "rustc": "string",
  "tool_version": "string",
  "adapters": [...],
  "policy": {...},
  "time_source": "system_clock"
}
```

## run outputs/index.json
```
{
  "files": [
    {"node_id": "id", "node_fingerprint": "...", "sha256": "...", "path": "nodes/<id>/outputs/file"}
  ]
}
```

## resolved_params.json
Resolved parameters for the node with deterministic key ordering.

## inputs/index.json
```
{
  "files": [
    {"path": "upstream/file", "sha256": "...", "from_node": "id", "from_node_fingerprint": "...", "from_output": "port"}
  ]
}
```

## outputs/index.json
```
{
  "files": [
    {"path": "file", "sha256": "...", "node_id": "id", "node_fingerprint": "..."}
  ]
}
```
