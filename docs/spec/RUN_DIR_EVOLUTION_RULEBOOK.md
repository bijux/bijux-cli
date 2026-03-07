# Run Directory Evolution Rulebook

- `manifest_version` is the authoritative run-dir format tag.
- Required file removal is breaking.
- Required file additions require defaults/backfill strategy.
- Unsupported versions must fail verify/doctor with precise diagnostics.
