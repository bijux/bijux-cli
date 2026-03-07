# Root Test Asset Migrations

This report records canonical scenario and helper assets moved out of top-level `tests/`.

## Moved To Evidence

- `tests/e2e/matrix.json` -> `evidence/battle/workflows/e2e_matrix.json`

## Moved To Crate-Local Test Fixtures

- `tests/integration_fixtures/minimal_consumer/README.md` -> `crates/bijux-dag-testkit/fixtures/minimal_consumer/README.md`

## Moved To Evidence Authoring

- `tests/integration_fixtures/minimal_consumer/dag.json` -> `evidence/authoring/examples/minimal_consumer.dag.json`
