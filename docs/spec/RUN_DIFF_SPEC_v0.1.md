# Run Diff Spec v0.1

## Scope

Run diff compares two run directories using manifest, graph fingerprint, node traces, and output payload indexes.

## Inputs

- `manifest.json` for run A and run B
- `graph.snapshot.json` for run A and run B
- node trace payloads under `nodes/*/trace.json`
- output index payloads under `nodes/*/outputs/index.json`

## Semantic Dimensions

- `manifest`
- `graph_fingerprint`
- `nodes`
- `outputs`

## Required Output Fields

- `manifest` (object)
- `graph_fingerprint` (object or null)
- `nodes` (object)
- `outputs` (object)
- `replay_equivalence.equivalent` (boolean)
- `replay_equivalence.reasons` (array)
- `replay_equivalence.reason_report` (object)
- `replay_equivalence.cause_groups` (object)

## Cause Group Contract

- `manifest_drift`
- `graph_semantics`
- `node_outcomes`
- `artifact_payload`

## Determinism Requirements

- repeated execution on identical run inputs MUST produce byte-identical JSON output
- node and output keys MUST be ordered deterministically

## Non-Goals

- wall-clock performance attribution
- policy recommendation beyond reported causes
