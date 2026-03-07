# Schema Evolution Rulebook

## Additive changes
Allowed when new fields are optional and defaults preserve existing semantics.

## Deprecation changes
Allowed when deprecated fields retain behavior and are documented with migration guidance.

## Breaking changes
Require graph schema version bump, compatibility fixture updates, and explicit release note entry.
