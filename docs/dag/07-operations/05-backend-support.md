# Backend Support

This document defines what operators can expect from each backend support class and where guarantees are intentionally bounded.

## Support classes

bijux-dag uses three support classes:
- `stable`: release-gated, continuously verified for required capability set.
- `bounded`: supported for defined scenarios with documented capability gaps.
- `unsupported`: available only for experimentation; no release guarantee.

## Operator expectations by backend class

### Stable

Operators can expect:
- complete run lifecycle reporting,
- artifact lineage sufficient for inspect/replay/diff,
- deterministic classification behavior within declared environment envelope,
- inclusion in release readiness CI gates.

### Bounded

Operators can expect:
- documented capability subset,
- explicit downgrade behavior when unsupported features are requested,
- inclusion in targeted compatibility lanes, not universal release gates.

Operators must not expect full equivalence with stable backends.

### Unsupported

Operators can expect:
- no compatibility promise,
- no replay/diff equivalence commitment,
- no regression response SLA.

## Backend family notes

Local shell family:
- strongest baseline for development and CI.
- bounded by host environment drift (shell/toolchain/locale).

Containerized family:
- stronger environment pinning when image is immutable.
- bounded by runtime engine differences and host kernel behavior.

Remote/managed execution family:
- may offer scale and isolation controls.
- bounded by provider APIs, scheduling behavior, and artifact egress policies.

## Non-equivalence and capability limits

Backend portability does not imply backend equivalence.

Common non-equivalence patterns:
- same graph identity, different timing/resource profile,
- replay possible but only `bounded` classification,
- artifact lifecycle available with reduced lineage detail on one backend.

When non-equivalence appears, operators must classify it explicitly and avoid stable-equivalence claims.

## Guarantees

- Support class semantics are explicit and auditable.
- Stable and bounded expectations are separated.
- Degraded guarantees are declared, not hidden.

## Non-guarantees

- Cross-backend wall-clock equivalence.
- Identical behavior outside declared capability envelope.
- Promotion safety when classifications are downgraded or unknown.

## Next reading

- [Adapters architecture](../05-system-architecture/05-adapters.md)
- [Portability architecture](../05-system-architecture/10-portability.md)
- [Replay semantics contract](../06-specification/07-replay-semantics.md)
