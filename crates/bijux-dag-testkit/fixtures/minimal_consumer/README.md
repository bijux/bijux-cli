# Minimal Consumer Fixture

This fixture contract represents the smallest external project shape exercised
through the `bijux-dag` command-line boundary. The canonical DAG is
`evidence/authoring/examples/minimal_consumer.dag.json`; this directory does
not keep a second copy that could drift.

## What The Fixture Proves

The canonical input is used by authoring and validation evidence consumers. It
checks that documented CLI commands accept a repository-external DAG without
depending on workspace-internal Rust APIs. It is not a package compilation
fixture and does not prove compatibility for every backend or artifact store.

## Run From The Repository Root

```sh
cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/authoring/examples/minimal_consumer.dag.json \
  --strict
cargo run -p bijux-dag-cli --bin bijux-dag -- run \
  evidence/authoring/examples/minimal_consumer.dag.json \
  --out artifacts/dag/minimal-consumer \
  --run-id minimal-consumer
```

## Maintenance

Keep the canonical DAG limited to features promised to ordinary CLI consumers.
When its path changes, update the evidence registry, ownership ledger,
consumer matrix, specifications, and handbook references together. A
feature-specific example belongs in the DAG handbook or a dedicated test
fixture. Generated run directories remain under `artifacts/`.
