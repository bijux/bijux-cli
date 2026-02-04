# Run Artifact Spec v0.1

## Run Directory Layout
```
run-<id>/
  manifest.json
  graph.snapshot.json
  nodes/
    <node_id>/
      trace.json
      stdout.log
      stderr.log
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

## outputs/index.json
```
{
  "files": [
    {"path": "file", "sha256": "..."}
  ]
}
```
