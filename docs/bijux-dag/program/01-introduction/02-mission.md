# Mission

Build and maintain a DAG execution system where behavior claims are evidence-backed, bounded, and reproducible.

## What this mission forces

This is intentionally a hard technical mission, not a values slogan. It constrains product and documentation decisions:

- if behavior cannot be classified through replay/diff evidence, it is not complete;
- if a feature expands scope but weakens guarantees, scope loses;
- if a claim cannot be bounded and tested, it is documentation noise.

## Design implications

The mission directly requires:

- identity-backed graph/run/artifact model,
- explicit run evidence surfaces,
- replay and diff as first-class control loops,
- narrow backend support language with declared capability envelopes.

This is why bijux-dag chooses deterministic evidence depth over orchestration breadth.

## Non-goals

- becoming a general orchestration platform for every scheduling policy and deployment topology,
- making universal portability claims across unsupported backend capability sets,
- accepting “best effort” semantics where contract-level behavior is required.

## Decision filter

A proposal is mission-aligned only if it improves at least one of:

- evidence quality,
- classification reliability,
- bounded portability clarity,
- operator decision confidence.

If it improves none, or weakens one of these, it should not ship.

## Guarantees you can verify

- Mission language in this repository is technical and scope-bounding, not roadmap marketing.
- Core docs can be traced to this mission through explicit replay/diff/identity boundaries.
- Feature justification can be evaluated against the decision filter above.

## Explicit limits

- The mission does not define implementation details; specs do.
- The mission does not promise support for every backend family.
- The mission does not replace maintainer judgment when principles conflict; it constrains that judgment.

## Next reading

- Problem framing and system intent: [What Is Bijux Dag](../01-introduction/01-what-is-bijux-dag.md)
- Engineering tradeoffs behind this mission: [Design Principles](../01-introduction/03-design-principles.md)
- Architectural realization of this mission: [System Overview](../05-system-architecture/01-system-overview.md)
- Operational support boundaries: [Backend Support](../07-operations/05-backend-support.md)
