# Forward Compatibility Notes for Envelope V2

## Purpose
Document non-breaking preparation constraints for a future envelope v2.

## Notes
- Keep `status` and `meta` semantics stable so parsers can branch by `meta.version`.
- Additive fields should be ignored by v1 consumers.
- Do not repurpose v1 fields with new meaning.
- Prefer introducing new nested objects over changing existing field types.

## Planned v2 directions
- Explicit `warnings` array for partial-success scenarios.
- Optional `links` object for machine-followable references.
- Structured `diagnostics` section compatible with `DiagnosticRecord`.
