# Compatibility Shim Policy Law

Compatibility shims and aliases are temporary by default and must continuously shrink.

Frozen requirements:

1. Live compatibility shims and aliases are tracked as generated artifacts.
2. Every remaining shim must include a classification, justification, and removal plan.
3. Every remaining alias must include a classification, justification, and removal plan.
4. Permanent compatibility shims are forbidden without explicit evidence.
5. Shim and alias counts are reported with before/after deltas against a baseline.

Evidence sources:

- `artifacts/status/compatibility_shim_inventory.json`
- `artifacts/status/compatibility_alias_inventory.json`
- `artifacts/status/live_compatibility_shims.json`
- `artifacts/status/live_compatibility_aliases.json`
- `artifacts/status/compatibility_shim_count_delta.json`
- `artifacts/status/compatibility_alias_count_delta.json`
