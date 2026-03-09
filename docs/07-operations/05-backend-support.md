# Backend Support

## Purpose
Define backend capability expectations, support tiers, and portability boundaries.

## Context
Backend support directly affects determinism, replay guarantees, and operational portability.

## Explanation
Support tier model:
- `stable`: actively supported and release-gated.
- `provisional`: supported for targeted scenarios with explicit caveats.
- `experimental`: available for development feedback, not release-critical guarantees.

Capability domains:
- execution lifecycle support (start/stop/exit classification).
- artifact persistence compatibility.
- timeout/cancellation enforcement.
- replay/diff evidence completeness.

Supported execution environment examples:
- local-shell runners on Linux/macOS (common stable baseline for development and CI).
- containerized runners where command/runtime surfaces are pinned and reproducible.
- provisional adapters where capability gaps are explicitly documented and tested.

Backend policy rules:
- stable tier backends must satisfy core run/artifact identity contracts.
- portability claims are bounded by shared capability subset.
- unsupported capability gaps must be documented as explicit limitations.

Operational governance:
- backend tier changes require changelog and documentation updates.
- release gates must verify stable tier coverage.
- regressions in stable capability surface block release readiness.

Operational recommendations:
- select one stable backend as release gate authority.
- use provisional/experimental backends for compatibility exploration, not primary release decisions.
- maintain a capability matrix artifact and update it with every backend behavior change.

## Examples
```text
Capability matrix sample:
backend: local-shell
tier: stable
supports: run lifecycle, artifacts, timeout, replay evidence
limits: environment-specific shell differences
```

```text
Portability statement example:
"Equivalent replay is guaranteed only across stable backends that share required capability set."
```

## Guarantees
- Support tiers and capability expectations are explicit.
- Portability promises are constrained by declared backend capabilities.
- Stable-tier backend regressions are operationally visible.
- Supported execution environment guidance is explicit and actionable.

## Limitations
- No guarantee of identical performance across backend families.
- Experimental backends are not release-grade by default.
- This document does not define every backend implementation detail.

## Related
- `docs/05-system-architecture/05-adapters.md`
- `docs/05-system-architecture/10-portability.md`
- `docs/07-operations/01-ci-integration.md`
- `docs/06-specification/07-replay-semantics.md`
