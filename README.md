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
cargo build -p bijux-dag-cli

# validate
cargo run -p bijux-dag-cli -- dag validate examples/hello.dag.json

# validate with explain
cargo run -p bijux-dag-cli -- dag validate examples/hello.dag.json --explain

# run
cargo run -p bijux-dag-cli -- dag run examples/hello.dag.json --out runs/

# run with selectors
cargo run -p bijux-dag-cli -- dag run examples/hello.dag.json --out runs/ --select tag:etl

# init
cargo run -p bijux-dag-cli -- dag init

# lint
cargo run -p bijux-dag-cli -- dag lint examples/hello.dag.json

# graph (dot)
cargo run -p bijux-dag-cli -- dag graph examples/hello.dag.json --format dot

# replay
cargo run -p bijux-dag-cli -- dag replay runs/run-<id> --out runs/

# diff
cargo run -p bijux-dag-cli -- dag diff runs/run-<id-a> runs/run-<id-b>

# export/import
cargo run -p bijux-dag-cli -- dag export runs/run-<id> --out run.json
cargo run -p bijux-dag-cli -- dag import run.json

# adapters
cargo run -p bijux-dag-cli -- dag adapters ls

# explain a run
cargo run -p bijux-dag-cli -- dag explain runs/run-<id>

# explain a node
cargo run -p bijux-dag-cli -- dag explain runs/run-<id> --node <node-id>

# cache verify
cargo run -p bijux-dag-cli -- dag cache verify
```

See `docs/CLI.md` for the command taxonomy, `docs/spec/` for formal definitions,
`docs/operations/README.md` for runtime usage, and `docs/ADAPTERS.md` / `docs/EFFECTS.md` for adapter/effects guidance.

## License
Apache-2.0. See `LICENSE` and `NOTICE`.

## Security
The `make security` target runs `cargo audit` to check dependencies for known vulnerabilities.
Install it once with:
```
cargo install cargo-audit
```
