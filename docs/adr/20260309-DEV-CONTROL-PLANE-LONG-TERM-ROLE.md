# Dev Control Plane Long-Term Role

Status: accepted
Owner: dev control-plane maintainers
Date: 2026-03-09

## Decision
`bijux-dev-dag` remains the long-term governance and verification control plane with constrained scope and durable command taxonomy.

## Consequences
- Governance logic is centralized and versioned.
- Helper and command surfaces remain contract-disciplined.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-DEV-DAG-LONG-TERM-ROLE.md
# ADR: Dev DAG Long-Term Role

## Status

Accepted

## Context

`bijux-dev-dag` grew into a broad command surface spanning release-critical, maintenance, advisory, and compatibility functions. Without explicit role boundaries, the surface risks bloat and operator confusion.

## Decision

1. Maintain explicit purpose classification for dev-dag commands.
2. Keep compact release-critical and maintenance command packs as primary entrypoints.
3. Treat advisory and legacy commands as non-primary surfaces.
4. Require explicit signal ownership for new dev-dag commands.

## Consequences

- Release-critical governance remains machine-stable and focused.
- Maintenance workflows stay available without polluting primary operator narratives.
- Surface growth is constrained by suite- and contract-enforced ownership boundaries.

## Enforcement

- `configs/suites/dev_dag_contraction_verification.json`
- `crates/bijux-dev-dag/tests/dev_dag_surface_guarantees_contracts.rs`

### SOURCE: 20260308-DEV-DAG-HELPER-CONTRACT-SURFACE.md
# ADR: Dev-Dag Helper Contract Surface

- Date: 2026-03-08
- Status: Accepted

## Context

The dev-dag helper modules (`repo`, `tooling`, `report`, and selected command helpers) mix stable workflow expectations with implementation glue. Without explicit boundaries, low-level helper changes can accidentally alter release-facing developer workflows.

## Decision

- Treat helper modules that resolve workspace root, write reports, and invoke tooling wrappers as public-ish contracts.
- Require direct in-file tests for helper modules, including very small modules.
- Keep generated helper health reports in `docs/reports/foundation/` and enforce them with contract tests.
- Keep command-family orchestration glue internal unless explicitly promoted through ADR.

## Public-ish helper contracts

- `repo/root.rs`
- `repo/layout.rs`
- `report/write.rs`
- `tooling/cargo.rs`
- `tooling/git.rs`
- `tooling/mod.rs`

## Internal glue (non-public contract by default)

- command-family wiring and dispatch internals in `commands/mod.rs`
- helper composition details not exposed through stable command outputs

## Consequences

- Helper behavior regressions are caught earlier by direct tests and helper fast suites.
- Review scope is clearer for changes that impact developer-facing release/evidence workflows.

### SOURCE: 20260308-INTERNAL-CONTRACT-DISCIPLINE.md
# ADR: Internal Contract Discipline

## Status

Accepted

## Context

Internal contracts span multiple crates and boundaries. Missing ownership, fixture links, docs links, or suite mapping weakens reliability and maintainability.

## Decision

1. Require direct tests and ownership for internal contracts.
2. Require docs/spec linkage for stable internal contracts.
3. Maintain generated contract-to-fixture and contract-to-suite mapping outputs.
4. Enforce drift detection through dedicated governance suites.

## Consequences

- Internal contract quality becomes auditable and comparable over time.
- Boundary regressions are caught earlier through explicit governance signals.
- Maintainers get clear ownership and review expectations.

## Enforcement

- `configs/policy/internal_contract_governance.json`
- `configs/suites/internal_contract_verification.json`
- `crates/bijux-dev-dag/tests/internal_contract_governance_contracts.rs`
