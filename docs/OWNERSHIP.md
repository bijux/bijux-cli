# Ownership model

## Spec ownership

The DAG spec (`docs/spec/*`) and compatibility fixtures are owned by repository maintainers of
`bijux-dag-core`.

## Runtime ownership

Runtime execution behavior and trace/manifests are owned by `crates/bijux-dag-runtime`.
Changes that alter node status semantics or manifest layout must update `docs/spec/*` and tests in `crates/bijux-dag-runtime/tests`.

## Artifact ownership

Artifact schema and serialization contracts are owned by `crates/bijux-dag-artifacts`.
Artifact layout changes require updates to:
- `docs/spec/RUN_ARTIFACT_SPEC_v0.1.md`
- golden/replay contract tests
- public API baselines if applicable.

## Policy ownership

Effect and policy documents (`docs/EFFECTS.md`, `docs/POLICY.md`) are the source-of-truth for
`--deny-*` behavior and must align with validator/error code definitions in `docs/spec/VALIDATION_RULES.md`.
