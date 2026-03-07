# Schema Field and Command Deprecation Policy

## Field deprecation
- Deprecations must include replacement guidance.
- Deprecated fields remain parseable within the compatibility window.
- Removal requires version bump and compatibility fixture updates.

## Command deprecation
- CLI deprecations follow `docs/spec/CLI_DEPRECATION_AND_ALIAS_POLICY.md`.
- Stable command removal requires explicit release-note migration guidance.
