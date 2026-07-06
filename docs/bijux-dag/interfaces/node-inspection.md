---
title: Node Inspection
audience: operators
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Node Inspection

Use `dag node` when the run already exists and one node needs a deeper evidence
read than `dag runs show`, `dag runs timeline`, or `dag runs explain-failure`
provide.

`dag node` is an explicit-path inspection helper. It remains outside the
default `bijux-dag --help` surface for `v0.4.0`, but it is repository-tested
and intended for focused operator diagnostics.

## Command

```bash
bijux-dag node <run_dir> --id <node_id>
```

Add `--json` when the result will be consumed by tooling.

## Evidence surfaced

- planned node fields from the persisted graph snapshot
- resolved params when `resolved_params.json` exists
- input and output artifact indexes when they exist
- terminal attempt number and per-attempt status history
- terminal stdout and stderr paths plus tail excerpts
- configured cache policy and observed cache result
- failure and lifecycle evidence from the node trace
- evidence gaps for missing optional files

## Notes

- pass an explicit run directory instead of relying on ambient workspace state
- treat `evidence_gaps` as a real diagnostic signal, not just missing
  decoration
- use `dag trace-node` only when raw trace payload debugging is the goal rather
  than operator inspection
