# Plugin SDK examples

This document provides contract-level examples for supported plugin patterns.

## Adapter SDK examples

### Local process adapter

- boundary: `TaskAdapter`
- capability: `local-process`
- execution mode: in-process or subprocess with typed result envelope

### Container task adapter

- boundary: `TaskAdapter`
- capability: `container-task`
- execution mode: container contract with image/policy restrictions

### Remote service adapter

- boundary: `TaskAdapter`
- capability: `remote-service`
- execution mode: external adapter with request/response provenance

## Artifact store SDK examples

### Filesystem artifact store

- boundary: `ArtifactStore`
- capability: `filesystem-store`
- contract: local read/write + conformance roundtrip verification

### Object storage artifact store

- boundary: `ArtifactStore`
- capability: `object-store`
- contract: typed key/value IO with integrity proof enforcement

## Observability exporter SDK examples

### Structured file exporter

- boundary: `ObservabilitySink`
- capability: `structured-file`
- contract: JSON line/event payload sink with stable schema

### OTLP-compatible exporter

- boundary: `ObservabilitySink`
- capability: `otlp-export`
- contract: trace/metric export mapping with deterministic category mapping
