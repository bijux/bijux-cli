# Compatibility Promise

## Scope
Defines compatibility commitments for pre-release and stable tracks.

## Tracks
- `0.x` pre-release: breaking changes may occur, but must be documented and migration-noted.
- `1.x+` stable: breaking changes require explicit major version increment and compatibility notes.

## Support window
Supported compatibility window is documented in `docs/COMPATIBILITY_WINDOW_v0.1.md`.

## Related tests
- `configs/schema/fixtures/compat/positive/*`
- `configs/schema/fixtures/compat/negative/*`

## Versioning and change policy
Support-window changes require release-policy update and compatibility matrix refresh.
