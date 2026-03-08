# Runtime Telemetry Schema

## Purpose

Define a stable runtime telemetry envelope for operator diagnostics and release verification.

## Canonical schema

- `configs/schema/operator/runtime_telemetry.schema.json`
- schema version: `runtime-telemetry/v0.1`

## Required coverage signals

- node-duration telemetry
- run-duration telemetry
- scheduler telemetry
- cache hit and miss telemetry
- replay telemetry
- diff telemetry
- prove/verify telemetry
- artifact IO telemetry
- backend capability telemetry
- failure, cancellation, and partial-rerun telemetry

## Compatibility guarantees

- stable required keys are backward compatible within `v0.1.x`
- forward schema versions are rejected until explicitly supported
