# Architecture Decision Records

Audience: maintainers.  
Owner: platform documentation guild.  
Status: historical.

ADRs capture durable decisions that changed architecture direction.

## Directory role

- Keep ADRs as historical artifacts with rationale and outcomes.
- Remove user guidance and operational narratives from ADRs.
- Do not duplicate root-level governance pages or reference material.

## Operationally relevant ADRs

- Documentation and evidence governance:
  - `20260308-documentation-truth-policy.md`
  - `20260308-evidence-minimalism.md`
  - `20260308-evidence-severity-rationalization.md`
- Runtime and boundary governance:
  - `20260309-runtime-contraction-governance.md`
  - `20260309-runtime-quarantine-rationale.md`
  - `20260309-app-crate-boundary.md`
  - `20260309-crate-boundary-governance.md`
  - `20260309-control-plane-service-migration-boundary.md`
- Release and operator guarantees:
  - `20260308-cli-stability-guarantees.md`
  - `20260308-schema-compatibility-guarantees.md`
  - `20260308-run-history-guarantees.md`
  - `20260308-inspect-guarantees.md`
  - `20260308-replay-planning-guarantees.md`

## Archived intermediate decisions

- Intermediate, superseded, and checkpoint ADRs are under `docs/adr/archive/`.
- These documents remain for traceability and should not be used as current policy.

## Naming rule

Use `YYYYMMDD-title.md`.
