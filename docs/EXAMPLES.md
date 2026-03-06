# Examples governance

## Contract examples

- `examples/hello.dag.json` is the canonical compatibility example.
- New examples must include stable behavior with no external non-deterministic dependencies.

## Ownership and review

- Changes to example graphs require review against:
  - policy compatibility
  - deterministic outputs
  - execution portability across local targets.

## Additional realistic examples

- `examples/etl-constant-to-shell.dag.json`
- `examples/cached-branched-report.dag.json`
- `examples/multi-output-artifact.dag.json`

## Usage policy

Examples are documentation-first assets; they should continue to parse and run with default local toolchain.
