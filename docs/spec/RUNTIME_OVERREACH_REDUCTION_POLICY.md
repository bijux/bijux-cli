# Runtime Overreach Reduction Policy

## Goal

Keep `bijux-dag-runtime` focused on deterministic execution, planning, state-machine legality, artifact integrity, and trust-critical boundaries.

## Rule

Runtime modules classified as `move` in `configs/policy/runtime_overreach_cleanup.json` must not become release-evidence requirements and must not expand runtime kernel authority.

## Enforcement

- Contract test: `crates/bijux-dev-dag/tests/runtime_overreach_contracts.rs`
- Report: `docs/reports/foundation/RUNTIME_OVERREACH_BEFORE_AFTER_REPORT.md`

## Scope decisions

- Keep: semantic lineage storage integrity required by artifact trust boundaries.
- Move: AI assist, workflow productization, ecosystem adoption/packaging, adaptive/cost models, federated/geo/HA scheduler surfaces, control-plane API in runtime, provenance compliance policy.
