# Schema Evolution Strategy

## Purpose
Define versioning rules for contract schema evolution.

## Strategy
- Envelope and manifest schemas are versioned, additive-first contracts.
- Backward-compatible additions: new optional fields only.
- Incompatible changes require a new versioned schema identifier.

## Versioning rules
- Current baseline: `v1` envelopes and `plugin manifest v2`.
- Minor releases may add optional fields and definitions.
- Future major releases may introduce `v2` envelopes or `v3` plugin manifests with notice.

## Compatibility requirements
- Existing required fields in `v1` remain required for the lifetime of `v1` support.
- Existing enum values remain valid for the lifetime of `v1` support.
- Schema changes must be accompanied by updated generated artifacts.
