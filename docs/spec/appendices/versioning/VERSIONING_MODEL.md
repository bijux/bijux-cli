# Versioning Model

## Versioned surfaces
- binary version: CLI/package version (`cargo` semver)
- crate API version: per crate semver and stability scope
- graph schema version: DAG `spec` field
- run-dir format version: manifest `manifest_version`
- export bundle version: bundle `export_bundle_version`

## Compatibility matrix authority
See `docs/reference/COMPATIBILITY_MATRIX.md`.

## Compatibility rules
- Additive schema fields: allowed if defaults preserve behavior.
- Deprecations: must include docs + fixture coverage.
- Breaking changes: require explicit version bump and negative fixtures.
- Silent reinterpretation of unsupported versions is forbidden.
