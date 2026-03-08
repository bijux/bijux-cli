# Specification documentation

Audience: implementers and maintainers.  
Owner: platform architecture and protocol maintainers.  
Status: stable.

This directory is the contract layer.  
All behavior statements in `spec/` should be normative and implementation-grade.

## Canonical contract clusters

- `TERMINOLOGY_AND_NAMING_CONTRACT.md`  
  Canonical vocabulary and naming governance with appendices in `appendices/terminology`.
- `VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md`  
  Canonical contract/version/schema governance with appendices in `appendices/versioning`.
- `CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md`  
  Configuration precedence, deprecation, and boundary rules with appendices in `appendices/config`.
- `CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md`  
  CLI surface, stability, compatibility, and taxonomy with appendices in `appendices/cli`.
- `BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md`  
  Backend, adapter, and protocol/runtime compatibility with appendices in `appendices/backend`.
- `BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md`  
  Performance evidence policy with appendices in `appendices/benchmark`.
- `EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md`  
  Evidence vocabulary, publication boundaries, and proof model with appendices in `appendices/evidence`.
- `RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md`  
  Runtime execution semantics and scheduler behavior with appendices in `appendices/runtime`.
- `SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md`  
  System reliability, invariants, introspection, and diagnostics with appendices in `appendices/system`.

## Directory role

- Keep only canonical contracts and their appendices.
- Do not keep explanatory narratives, how-to guides, or historical status commentary.
- De-duplicate overlapping documents; each contract family should have one canonical root file.
- If behavior belongs in a user guide, move it to `user/`.

## Stability contract

Every file should declare whether it is stable, evolving, or historical.

## Shape targets

- Root cluster target: one canonical contract file per cluster.
- Appendices target: supporting detail, implementation examples, and migration history.
- Canonical contract rule: each contract should include a `what this spec is not` section.
- No single spec file should grow beyond practical readability thresholds (~500 lines).
