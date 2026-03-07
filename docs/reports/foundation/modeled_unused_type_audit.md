# Modeled Unused Type Audit

This audit tracks modeled-but-unused candidates for move/delete decisions.

## Candidates
- simulated-only metadata structs not referenced by runtime execution path
- placeholder compatibility payload wrappers with no consumer command path

## Action policy
- move to evidence/governance surfaces if informational only
- delete when no command, test, or report consumes the type
- keep only when covered by contract tests
