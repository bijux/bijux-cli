# Operations

## Running
Use the CLI to validate and run DAGs:
```
bijux dag validate examples/hello.dag.json
bijux dag run examples/hello.dag.json --out runs/
```

## Selectors
Run subsets with selectors:
```
bijux dag run dag.json --out runs/ --select tag:etl
bijux dag run dag.json --out runs/ --exclude tag:gpu
bijux dag run dag.json --out runs/ --select kind:shell
bijux dag run dag.json --out runs/ --select id:etl_
```
Filtered nodes are marked `skipped` with reason `filtered`.

## Init & Lint
Create a starter DAG:
```
bijux dag init
```

Lint suggestions:
```
bijux dag lint dag.json
```

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
- `inputs/index.json` (materialized inputs + provenance)

## Replay
Replay a run from its embedded graph snapshot:
```
bijux dag replay runs/run-123 --out runs/
```

## Diff
Compare two runs:
```
bijux dag diff runs/run-123 runs/run-456
```

## Graph Export
```
bijux dag graph dag.json --format dot
```

## Adapters
List adapters:
```
bijux dag adapters ls
```

## Doctor
Check environment health:
```
bijux dag doctor
```

## Export/Import
```
bijux dag export runs/run-123 --out export.json
bijux dag import export.json
```

## Compatibility Suite
```
bijux dag compat
```

## Caching
Cache modes:
- `off`
- `read`
- `readwrite`

```
bijux dag run dag.json --out runs/ --cache readwrite
```

Verify cache integrity:
```
bijux dag cache verify
```

Verify cache integrity including a remote cache directory:
```
bijux dag cache verify --remote /path/to/remote-cache
```

## Resources
Use `--jobs` and `--cpu-budget` to control concurrency and CPU aggregate.

## Inputs Materialization
```
bijux dag run dag.json --out runs/ --materialize-inputs copy
```

## Failures
Failed nodes produce a failure object in `trace.json`. Downstream nodes
are marked `skipped`. Cancelled runs set manifest status to `cancelled`.
