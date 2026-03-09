# Compatibility window for v0.1

## Statement

For spec `v0.1`, bijux-dag guarantees backward compatibility for canonical parsing, validation diagnostics contract, and run artifact schema throughout all `0.1.x` releases.

## Included guarantees

- Canonical JSON generation remains stable for unchanged graph semantics.
- Validation error and warning codes remain stable.
- Runtime manifest and node trace schema versions remain stable.

## Exclusions

- Internal implementation details and non-contract debug output.
- Experimental commands not listed in the CLI compatibility contract.
