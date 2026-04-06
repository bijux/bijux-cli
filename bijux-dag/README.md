# bijux-dag

DAG-specific documentation, evidence assets, and configs live in this directory.

DAG crates are part of the root workspace under:

- `crates/bijux-dag-core`
- `crates/bijux-dag-artifacts`
- `crates/bijux-dag-runtime`
- `crates/bijux-dag-app`
- `crates/bijux-dag-cli`
- `crates/bijux-dag-testkit`
- `crates/bijux-dev-dag`

Run from repository root:

```bash
cargo check --workspace
cargo run -p bijux-dag-cli -- dag --help
make dag-test
make dag-contracts
```
