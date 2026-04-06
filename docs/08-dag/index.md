# DAG Documentation

The DAG subsystem is maintained in this repository and documented under `bijux-dag/docs/`.

## Entry points

- [DAG overview](../../bijux-dag/README.md)
- [What is bijux-dag](../../bijux-dag/docs/01-introduction/01-what-is-bijux-dag.md)
- [Getting started](../../bijux-dag/docs/02-getting-started/01-installation.md)
- [User guide](../../bijux-dag/docs/03-user-guide/01-graph-schema-and-validation.md)
- [Specification](../../bijux-dag/docs/06-specification/01-object-model.md)
- [Operations](../../bijux-dag/docs/07-operations/01-ci-integration.md)
- [Development](../../bijux-dag/docs/08-development/01-architecture-principles.md)

Run DAG commands from repository root:

```bash
cargo run -p bijux-dag-cli -- dag --help
make dag-test
make dag-contracts
```
