# Portability

## Purpose
Define portability architecture design, guarantees, and non-goals.

## Context
Portability determines how workflow context and outputs move across environments while preserving trust.

## Explanation
Portability architecture model:
- bundle export packages transferable workflow context
- bundle import restores that context in a target environment
- replay and diff validate behavioral equivalence or bounded divergence

Bundle portability design:
- portability is identity-aware and evidence-aware
- portability verification requires replay/diff, not assumption

Backend equivalence boundaries:
- supported adapters define the meaningful portability envelope
- unsupported adapter features are explicit non-equivalence zones

Architecture tradeoff:
- strict semantic preservation is prioritized over broad but ambiguous compatibility claims

System non-goals (deliberate omissions):
- universal backend parity regardless of support boundaries
- performance identity across heterogeneous runtime environments
- speculative portability claims without validation workflows

## Examples
```text
Portability validation flow:
source run -> bundle export -> target import -> replay -> diff -> release decision
```

```mermaid
graph LR
  A[Source Environment] -->|Export Bundle| B[Bundle Artifact]
  B -->|Import| C[Target Environment]
  C --> D[Replay]
  D --> E[Diff]
  E --> F[Portability Decision]
```

## Guarantees
- Portability is documented as a validated workflow, not a blanket promise.
- Backend support boundaries are explicit architecture constraints.

## Limitations
- Portability does not guarantee identical timing or resource behavior.
- Deep bundle schema internals belong to specification docs.

## Related
- `docs/05-system-architecture/05-adapters.md`
- `docs/03-user-guide/08-bundles-and-portability.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/03-artifact-model.md`
