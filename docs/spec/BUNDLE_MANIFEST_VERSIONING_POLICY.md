# Bundle Manifest Versioning and Migration Policy

## Scope

Defines version evolution for graph/run/artifact bundle formats.

## Current supported bundle version

- `export-bundle/v0.1`

## Compatibility rules

- Backward compatibility is required for all supported prior bundle versions.
- Unsupported bundle versions must fail import with explicit diagnostics.
- Migration behavior must be deterministic and idempotent.

## Migration policy

- `import --verify-only` must run structural and invariant checks without mutating runtime records.
- Migration tooling may add non-semantic metadata but must not alter canonical identities promised by contract.

## Release governance

- Release verification must include bundle conformance suites and backward-compatibility fixture checks.
