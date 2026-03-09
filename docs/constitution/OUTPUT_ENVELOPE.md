# Output Envelope

## Purpose
Define the stable machine-readable success envelope.

## Scope
This document governs JSON and YAML success payload contracts.

## Core Concepts
- Success payloads are versioned and stable for automation.

## Invariants
- Machine-readable success responses use this shape:
  - `status`: fixed string `"ok"`
  - `data`: command-specific payload object or array
  - `meta.command`: canonical command path
  - `meta.timestamp`: RFC 3339 timestamp
  - `meta.version`: envelope version identifier
- `--pretty` affects rendering only, not field semantics.
- `--format json --no-pretty` produces compact JSON suitable for pipes.

## Failure Modes
- If requested format cannot be emitted, return an encoding/serialization error.

## Schema Artifact
- JSON Schema (v1): `docs/constitution/schemas/output-envelope-v1.schema.json`

## Design Rationale
- Stable envelopes decouple script contracts from text presentation.

## Non-Goals
- Defining all command-specific `data` fields in one file.
