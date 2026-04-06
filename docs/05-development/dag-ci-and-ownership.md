# DAG CI And Ownership

`bijux-core` keeps DAG runtime ownership explicit without splitting repository roots.

## DAG ownership map

- Crates: `crates/bijux-dag-*`, `crates/bijux-core-dev`
- Config: `configs/dag/`
- Automation modules: `makes/`
- DAG documentation and evidence assets: `docs/dag/`, `evidence/dag/`
- DAG GitHub workflows: `.github/workflows/dag-*.yml`

## Local verification path

```bash
./crates/bijux-core-dev/scripts/verify-workspace-layout.sh
cargo check --workspace --all-targets
make dag-test
make dag-contracts
```

## CI scope policy

DAG workflows are path-scoped to DAG-owned files so CLI-only edits do not trigger DAG-heavy pipelines.

When changing DAG crate boundaries, config paths, or DAG make modules, update both:

- `.github/workflows/dag-*.yml`
- this document
