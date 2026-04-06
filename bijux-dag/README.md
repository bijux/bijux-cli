# bijux-dag

DAG-specific documentation and evidence assets live in this directory.

DAG config authority lives at `configs/dag/`.

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
cargo run -p bijux-dag-cli --bin bijux-dag -- dag --help
make dag-test
make dag-contracts
```
