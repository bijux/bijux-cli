# Platform sustainability governance

## Roadmap governance

Roadmap progression requires:
- contract coverage
- documentation coverage
- operational readiness

## Lifecycle governance

Every experimental capability must have a decision deadline:
- promote to preview/stable
- deprecate
- remove

No feature may remain undecided indefinitely.

## Acceptance board

Preview-to-stable promotion is governed by a platform acceptance board with explicit criteria and sign-off records.

## Ownership and stewardship

Sustainability requires:
- subsystem ownership mapping
- review routing ownership
- long-term maintenance commitments

## Non-negotiable invariants

The platform invariant catalog is mandatory and includes determinism, immutability, isolation, and policy finality constraints.

## Integrated verification lane

`platform-integrated-verification` is the mandatory final lane combining evidence for:
- multi-tenant isolation
- HA scheduler behavior
- policy enforcement
- backend execution conformance
- artifact lineage integrity
- compatibility governance

The lane must fail closed on missing domain evidence.
