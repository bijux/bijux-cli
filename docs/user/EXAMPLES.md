# Examples governance

## Contract examples

- `evidence/authoring/examples/hello.dag.json` is the canonical compatibility example.
- New examples must include stable behavior with no external non-deterministic dependencies.

## Ownership and review

- Changes to example graphs require review against:
  - policy compatibility
  - deterministic outputs
  - execution portability across local targets.

## Additional realistic examples

- `evidence/authoring/examples/etl-constant-to-shell.dag.json`
- `evidence/authoring/examples/cached-branched-report.dag.json`
- `evidence/authoring/examples/multi-output-artifact.dag.json`
- `evidence/authoring/examples/replay-heavy-branching.dag.json`
- `evidence/authoring/examples/failure-heavy-retry.dag.json`

## Usage policy

Examples are documentation-first assets; they should continue to parse and run with default local toolchain.
