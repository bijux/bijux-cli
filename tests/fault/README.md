# Fault resilience model

This suite defines fault classes and expected behavior under failure.

## Fault classes

- filesystem: permission denied, disk pressure simulation, trace write failure, index write failure
- subprocess: non-zero exit, malformed output, hang/timeout, external kill
- environment: missing required variables
- config: malformed and version-incompatible configuration
- corruption: manifest tampering, missing trace, stale cache metadata mismatch
- concurrency: run-id collision and latest alias race
- artifact integrity: no silent half-valid success outputs

## Principles

- failures must be explicit and machine-detectable
- partial artifacts must carry failure status, never silent success markers
- verification commands must detect corruption and stale metadata

## Resume policy

Current product behavior has no run-resume contract. Recovery is replay-driven.
