# Schema Drift Diagnostics Report

This report defines drift classes and mandatory response paths.

| Drift class | Trigger | Response |
| --- | --- | --- |
| Hash drift | Frozen schema hash changed | Block merge until compatibility review and changelog update |
| Fixture drift | Compatibility fixture missing or malformed | Block merge and restore fixture coverage |
| Command drift | Stable JSON command surface changed | Block merge and update CLI contract with explicit migration note |
| Documentation drift | Schema policy pages missing | Block merge and restore governance surfaces |

## Linked controls

- `docs/spec/UNIFIED_SCHEMA_VERSIONING_POLICY.md`
- `docs/spec/SCHEMA_EVOLUTION_POLICY.md`
- `docs/spec/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md`
- `docs/spec/SCHEMA_FORWARD_COMPATIBILITY_LIMITATIONS.md`
