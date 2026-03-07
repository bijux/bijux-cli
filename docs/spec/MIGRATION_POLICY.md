# Migration Policy

## Supported migration modes
- Automatic: no-op format-preserving migrations (`from == to`).
- Manual: operator-managed rewrite with explicit report output.
- Unsupported: cross-major format jumps.

## Current support boundary
This repository currently supports no-op migration assertions and explicit rejection for unsupported migrations.
No broad automatic schema/run/export migration is claimed.

## Migration report format
Migration reports must include:
- source version
- target version
- changed fields
- dropped/unrepresentable fields
- status (`no-op` / `applied` / `rejected`)
