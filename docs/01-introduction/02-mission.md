# Mission

State the enduring mission of bijux-dag and the operational outcomes it targets.

The mission constrains product decisions, prioritization, and documentation scope.

## Explanation
Mission statement:
Build a deterministic, inspectable, and portable DAG execution system that lets teams explain workflow behavior with evidence instead of assumption.

Operationally, the mission means:
- execution behavior should be explicit rather than implicit
- run outcomes should be auditable through identity and provenance
- replay and diff should be routine operational tools
- documentation should prioritize truthful guarantees over broad claims

Long-term design goals:
- make run-to-artifact lineage reliable enough for incident diagnosis and release decisions.
- make replay and diff a default quality loop in development and operations.
- keep user-facing guarantees narrow, testable, and durable across repository evolution.
- preserve portability through explicit backend capability contracts, not implicit promises.

The mission excludes common failure modes in workflow platforms:
- opaque "best effort" semantics without contract boundaries
- feature sprawl that increases uncertainty instead of reliability
- documentation that mixes guarantees with speculative future designs

Explicit non-goals:
- becoming a general-purpose orchestration platform with every scheduling policy variant.
- claiming universal cross-environment equivalence without capability and evidence checks.
- optimizing for maximum feature count at the expense of diagnosability.

This mission guides documentation quality directly:
- user docs answer practical operational questions
- architecture docs explain current system boundaries
- specification docs define exact guarantees and limitations

Why deterministic execution matters:
- release confidence: teams can compare candidate runs against known baselines.
- incident response: failures can be traced using run and artifact identities.
- governance: contract language can be anchored to repeatable behavior, not anecdotes.

## Examples
```text
Mission test for a proposed feature:
1. Does it improve deterministic control?
2. Does it improve inspectability?
3. Does it preserve or improve portability boundaries?
4. Can the resulting behavior be documented as a concrete guarantee?
```

```text
Mission rejection example:
- proposal: add backend-specific behavior that bypasses normalized outcome classification
- result: reject or redesign because inspectability and comparability would degrade
```

## Guarantees
- The mission is stable and decision-driving.
- Mission language is technical and measurable.
- Documents in this tree are expected to align with this mission.

## Limitations
- The mission is not a full product roadmap.
- The mission does not promise immediate support for every backend or workflow pattern.

## Related
- `docs/01-introduction/01-what-is-bijux-dag.md`
- `docs/01-introduction/03-design-principles.md`
- `docs/05-system-architecture/01-system-overview.md`
- `docs/07-operations/05-backend-support.md`
