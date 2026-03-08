# Ownership model

Audience: maintainers.
Owner: governance and architecture leads.
Status: stable.

## Spec ownership

The DAG spec (`docs/spec/*`) and compatibility fixtures are owned by repository maintainers of `bijux-dag-core`.
Runtime execution semantics and manifests are owned by `crates/bijux-dag-runtime`.
Artifact schema and serialization contracts are owned by `crates/bijux-dag-artifacts`.

## Runtime ownership

Runtime behavior changes that alter node status semantics or manifest layout must update `docs/spec/*` and tests in `crates/bijux-dag-runtime/tests`.

## Artifact ownership

Artifact layout changes require updates to:

- `docs/spec/RUN_ARTIFACT_SPEC_v0.1.md`
- golden/replay contract tests
- public API baselines where applicable

## Operational and delivery ownership

- Spec and validation contracts: `bijux-dag-core` maintainers
- Runtime and policy execution: `bijux-dag-runtime` maintainers
- CLI contracts and output schemas: `bijux-dag-cli` and `bijux-dag-app` maintainers
- Repository governance and release checks: `bijux-dev-dag` maintainers
- Plugin and DSL extension contracts: `bijux-dag-runtime` and `bijux-dag-core` maintainers

## Policy and effect ownership

Policy source-of-truth for deny-list flags is now in `docs/SECURITY.md`.
Policy changes must align with validator/error-code behavior in:

- `docs/spec/VALIDATION_RULES.md`
- `docs/spec/POLICY_CONTRACT.md`

## Governance process

Every ownership handoff updates this document and related contract docs, then maps to release evidence.
