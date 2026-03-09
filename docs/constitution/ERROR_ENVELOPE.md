# Error Envelope

## Purpose
Define the stable machine-readable error envelope.

## Scope
This document governs structured error payloads across JSON and YAML formats.

## Core Concepts
- Errors have a stable envelope even when message text changes.

## Invariants
- Machine-readable errors use this shape:
  - `status`: fixed string `"error"`
  - `error.code`: stable symbolic code
  - `error.message`: user-readable summary
  - `error.category`: one of `usage`, `validation`, `plugin`, `internal`
  - `error.details`: optional object with structured context
  - `meta.version`: envelope version identifier
- Human-readable text mode may render a textual equivalent but must preserve semantics.

## Failure Modes
- Malformed envelope emission is treated as an internal failure.

## Schema Artifact
- JSON Schema (v1): `docs/constitution/schemas/error-envelope-v1.schema.json`

## Design Rationale
- Stable error envelopes preserve automation compatibility while allowing text improvements.

## Non-Goals
- Freezing exact phrasing of `error.message`.
