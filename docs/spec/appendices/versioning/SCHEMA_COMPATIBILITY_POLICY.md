# Schema compatibility policy

Schemas under `configs/schema/` are the source of truth for wire contracts.

## Compatibility classes
- Additive: adding optional fields, expanding enums with backward-safe defaults, adding optional objects.
- Breaking: removing fields, changing field types, making optional fields required, narrowing enums, changing required semantics.

## Versioning
- Breaking changes require a new schema version and migration notes.
- Additive changes remain within the same version only when old clients continue to parse and operate correctly.

## Fixture policy
- Each schema version must include positive and negative fixtures.
- Negative fixtures must include unknown fields, invalid enum values, malformed references, and invalid path shapes.
