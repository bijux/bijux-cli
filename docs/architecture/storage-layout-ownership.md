# Storage Layout Ownership

## Purpose

Define which module owns each storage path decision to prevent ad-hoc path
construction spread across runtime modules.

## Ownership Table

| Path family | Owner |
| --- | --- |
| run dir root and node paths | `RunDir` in artifacts crate |
| runtime run writes | runtime `engine` + `ArtifactStore` |
| cache entry paths | runtime `CacheStore` |
| export/import bundle paths | app/export surfaces (outside runtime execution loop) |

## Rules

- Runtime modules must not invent storage path strings when an owner API exists.
- New persisted artifacts require owner assignment in this document and in
  `docs/spec/STORAGE_CONTRACT.md`.
