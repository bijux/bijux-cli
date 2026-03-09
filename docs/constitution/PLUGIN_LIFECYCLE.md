# Plugin Lifecycle

## Canonical Lifecycle States

`bijux-cli` tracks plugins with one canonical lifecycle state:

1. `discovered`: plugin artifacts were found on disk but not yet parsed.
2. `validated`: manifest and compatibility checks passed.
3. `installed`: plugin is registered and available for activation.
4. `enabled`: plugin is active and routeable.
5. `disabled`: plugin is installed but not active.
6. `broken`: plugin failed validation or runtime loading and is quarantined.
7. `incompatible`: plugin cannot run with the current `bijux-cli` compatibility range.

These states are stable contract terms for registry persistence and diagnostics.

## Manifest Versioning Policy

- Manifest schema uses explicit semantic versioning at `manifest_version`.
- `v1` manifests are parsed by `PluginManifestV1`.
- `bijux-cli` must reject unknown major manifest versions.
- Minor/patch additions in `v1` must remain backward compatible through optional fields.

## Plugin Kind Policy

Supported plugin kinds in v1:

- `delegated`
- `python`
- `external-exec`

Reserved but not yet executable in v1:

- `native`

`native` is schema-valid for forward compatibility but execution is intentionally stubbed and rejected by runtime validation until a dedicated ABI and sandbox contract is published.
