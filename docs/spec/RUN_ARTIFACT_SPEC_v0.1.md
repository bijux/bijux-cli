# Run Artifact Spec v0.1

## Run Directory Layout
```
run-<id>/
  manifest.json
  graph.snapshot.json
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

## resolved_params.json
Resolved parameters for the node with deterministic key ordering.

## inputs/index.json
```
{
  "files": [
    {"path": "upstream/file", "sha256": "...", "from_node": "id"}
  ]
}
```

## outputs/index.json
```
{
  "files": [
    {"path": "file", "sha256": "..."}
  ]
}
```
