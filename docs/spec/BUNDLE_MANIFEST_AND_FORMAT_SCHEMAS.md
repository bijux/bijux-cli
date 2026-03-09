# BUNDLE MANIFEST AND FORMAT SCHEMAS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/EXPORT_BUNDLE_EVOLUTION_RULEBOOK.md
# Export Bundle Evolution Rulebook

- `export_bundle_version` is required in exported bundles.
- Additive metadata is allowed with backward-compatible defaults.
- Structural changes require version bump and compatibility fixtures.
- Import must reject unsupported versions explicitly.
