# Artifact Storage Lifecycle Telemetry Report

Generated telemetry summary for artifact lifecycle health.

## Tracked signals

- lifecycle roundtrip failure count
- retention and GC decision mismatch count
- index consistency anomaly count
- checksum mismatch count
- corruption detection event count
- recovery action invocation count

## Source of truth

- `evidence/cache/artifact_lifecycle/regression_corpus.json`
- `configs/suites/artifact_storage_lifecycle_stress.json`
