# DAG Documentation

The DAG subsystem is maintained in this repository with crates under `crates/bijux-dag-*` and docs under `docs/dag/`.

## Entry points

- [DAG overview](../../bijux-dag/README.md)
- [What is bijux-dag](../dag/01-introduction/01-what-is-bijux-dag.md)
- [Getting started](../dag/02-getting-started/01-installation.md)
- [User guide](../dag/03-user-guide/01-graph-schema-and-validation.md)
- [Specification](../dag/06-specification/01-object-model.md)
- [Operations](../dag/07-operations/01-ci-integration.md)
- [Development](../dag/08-development/01-architecture-principles.md)

Run DAG commands from repository root:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- dag --help
make dag-test
make dag-contracts
```
