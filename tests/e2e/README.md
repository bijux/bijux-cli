# End-to-end suite

This directory is a first-class product-level e2e suite.

## Scenario taxonomy

- `happy_path`
- `failure`
- `replay`
- `cache`
- `selection`
- `compat`
- `import_export`
- `container`
- `policy`

## Scenario contract

Each scenario must define:

- input graph fixture
- command sequence
- expected exit codes
- expected manifest and trace assertions

## Binary boundary

E2E scenarios are the only test family allowed to shell out to production binaries.

## Matrix execution

Run matrix via:

- `cargo run -p bijux-dev-dag -- e2e-matrix`
