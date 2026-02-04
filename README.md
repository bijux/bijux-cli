# bijux-dag

A minimal, deterministic DAG runner and artifact system written in Rust. This repo provides a JSON DAG IR, validation rules, a runtime with stable scheduling, and a CLI for validating and running DAGs into reproducible run directories.

## Purpose
- Define a compact DAG IR that is strict and canonicalizable.
- Provide deterministic execution order with stable artifact outputs.
- Make runs easy to inspect and explain.

## Refusal List
This project does not:
- Execute untrusted code without explicit user intent.
- Perform network operations by default.
- Allow unknown JSON fields in DAG definitions.
- Accept non-deterministic scheduling.

## Quickstart
1. Build the CLI.
2. Validate a DAG.
3. Run a DAG and inspect artifacts.

Example commands (assuming `cargo` is available):
```
# test
make test

# lint
make lint

# security
make security

# build
cargo build -p bijux_dag_cli

# validate
cargo run -p bijux_dag_cli -- validate examples/hello.dag.json

# validate with explain
cargo run -p bijux_dag_cli -- validate examples/hello.dag.json --explain

# run
cargo run -p bijux_dag_cli -- run examples/hello.dag.json --out runs/

# run with selectors
cargo run -p bijux_dag_cli -- run examples/hello.dag.json --out runs/ --only-tag etl

# replay
cargo run -p bijux_dag_cli -- replay runs/run-<id> --out runs/

# diff
cargo run -p bijux_dag_cli -- diff runs/run-<id-a> runs/run-<id-b>

# export/import
cargo run -p bijux_dag_cli -- export runs/run-<id> --format json > run.json
cargo run -p bijux_dag_cli -- import run.json

# adapters
cargo run -p bijux_dag_cli -- adapters ls

# explain a run
cargo run -p bijux_dag_cli -- explain runs/run-<id>

# explain a node
cargo run -p bijux_dag_cli -- explain runs/run-<id> --node <node-id>

# cache verify
cargo run -p bijux_dag_cli -- cache verify
```

See `docs/spec/` for formal definitions, `docs/operations/README.md` for runtime usage,
and `docs/architecture/ADAPTERS.md` / `docs/architecture/EFFECTS.md` for adapter/effects guidance.

## License
Apache-2.0. See `LICENSE` and `NOTICE`.

## Security
The `make security` target runs `cargo audit` to check dependencies for known vulnerabilities.
Install it once with:
```
cargo install cargo-audit
```
