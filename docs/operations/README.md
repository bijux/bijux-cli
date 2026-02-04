# Operations

## Running
Use the CLI to validate and run DAGs:
```
bijux-dag validate examples/hello.dag.json
bijux-dag run examples/hello.dag.json --out runs/
```

## Selectors
Run subsets with tags:
```
bijux-dag run dag.json --out runs/ --only-tag etl
bijux-dag run dag.json --out runs/ --skip-tag gpu
```
Filtered nodes are marked `skipped` with reason `filtered`.

## Artifacts
Each run directory contains:
- `manifest.json` (run metadata)
- `graph.snapshot.json` (canonical graph + fingerprint)
- `run.log.jsonl` (event stream)
- `nodes/<id>/` per-node artifacts

Per-node layout:
- `trace.json` (status, fingerprint, cache proof, resolved params, failure)
- `stdout.log` / `stderr.log`
- `outputs/index.json` (files + sha256 + provenance)

## Replay
Replay a run from its embedded graph snapshot:
```
bijux-dag replay runs/run-123 --out runs/
```

## Diff
Compare two runs:
```
bijux-dag diff runs/run-123 runs/run-456
```

## Adapters
List adapters:
```
bijux-dag adapters ls
```

## Export/Import
```
bijux-dag export runs/run-123 --format json
bijux-dag import export.json
```

## Caching
Cache modes:
- `off`
- `read`
- `readwrite`

```
bijux-dag run dag.json --out runs/ --cache readwrite
```

Verify cache integrity:
```
bijux-dag cache verify
```

## Resources
Use `--jobs` and `--cpu-budget` to control concurrency and CPU aggregate.

## Failures
Failed nodes produce a failure object in `trace.json`. Downstream nodes
are marked `skipped`. Cancelled runs set manifest status to `cancelled`.
