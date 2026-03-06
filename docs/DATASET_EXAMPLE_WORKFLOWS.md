# Dataset-centric example workflows

## Sales mart publication

1. Materialize partitioned outputs.
2. Validate dataset quality contract.
3. Publish dataset version.
4. Promote latest approved version.

## Downstream consumption

A consumer workflow can require:
- stable dataset version for reproducibility, or
- latest approved version for freshness-sensitive reporting.

## Readiness-gated scheduling

Schedule execution proceeds only when required dataset readiness gates are accepted.

## Replay-safety expectation

Dataset-based replay remains deterministic by pinning dataset version references and preserving mapping evidence to produced artifacts.
