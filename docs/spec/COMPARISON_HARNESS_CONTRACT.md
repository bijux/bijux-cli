# Comparison Harness Contract

## Purpose
Comparisons are for evidence-driven analysis of:
- correctness behavior
- operator ergonomics
- performance shape
- observability shape

Comparisons are not marketing claims.

## Initial external subset
- Dagster
- Prefect
- Argo Workflows

## Canonical scenarios
- chain
- diamond
- retry-timeout
- cache-reuse-shape
- replay-equivalence
- failure-propagation
- determinism
- operator-inspectability
- failure-diagnostics
- scheduler-tiny-tasks-overhead
- artifact-inspectability

## Comparable vs non-comparable
- Comparable:
  - terminal outcomes
  - retry/failure propagation classes
  - timeline and inspect surfaces
  - relative scheduler overhead trends under same scenario shape
- Not comparable:
  - absolute wall time across different host/container setups
  - feature areas one engine does not support natively
  - claims outside committed scenario scope

## Evidence policy
- Public comparison statements must cite committed harness artifacts in `evidence/compare/`.
- Interpretations must be separated from raw facts.
- Claims using “better”, “faster”, or “superior” require scenario-scoped evidence references.
