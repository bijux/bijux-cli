# Graph Diff Spec v0.1

## Scope

Graph diff compares two DAG definitions at canonical semantic level.

## Inputs

- canonical graph bytes for graph A
- canonical graph bytes for graph B
- graph fingerprints for graph A and graph B

## Classification

- `semantic_change`: canonical graph bytes differ
- `cosmetic_only`: raw source text differs but canonical graph bytes are equal
- `equivalent`: canonical graph bytes are equal and fingerprints are equal

## Required Output Fields

- `equivalent` (boolean)
- `graph_fingerprint` (object or null)
- `reason_report.summary` (string)
- `cause_groups` (object)

## Determinism Requirements

- identical inputs MUST produce byte-identical JSON output
- field ordering MUST remain stable
- cause group naming MUST remain stable across patch releases

## Non-Goals

- runtime timing comparison
- resource usage attribution
- backend conformance evaluation
