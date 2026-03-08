# Glossary

Audience: operators and maintainers.  
Owner: platform documentation guild.  
Status: stable.

## Core terms

- `graph`: canonical DAG definition and identity surface.
- `run`: immutable execution record derived from a graph under runtime conditions.
- `artifact`: materialized output with content fingerprint and provenance links.
- `replay`: re-execution or re-materialization workflow with explicit fidelity checks.
- `proof`: evidence-backed claim that a behavior or trust property is verified.
- `fidelity`: degree to which replayed behavior is equivalent to source run semantics.

## Governance and scope terms

- `repository governance`: command and report orchestration in `bijux-dev-dag`.
- `local deterministic execution`: core runtime behavior shipped for product guarantees.
- `adapter execution`: supported node execution via local/container/external adapter paths.
- `operator diagnostics`: inspect/explain/status outputs for operators.
- `simulated distributed semantics`: modeled-only distributed surfaces that are not default shipped product capabilities.
- `experimental runtime surface`: incubating runtime modules under explicit quarantine and expiration criteria.
- `speculative runtime surface`: modeled/internal surfaces not treated as stable product scope.
