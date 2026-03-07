# First Hour with bijux-dag

## 1. Check binary and capability surface
```sh
cargo run -p bijux-dag-cli -- dag version --json
cargo run -p bijux-dag-cli -- dag capabilities --json
```

## 2. Validate and execute a DAG
```sh
cargo run -p bijux-dag-cli -- dag validate evidence/authoring/patterns/minimal.json --strict --json
cargo run -p bijux-dag-cli -- dag run evidence/authoring/patterns/minimal.json --out runs --run-id first-hour --json
```

## 3. Inspect and verify run artifacts
```sh
cargo run -p bijux-dag-cli -- dag runs inspect first-hour --root runs --json
cargo run -p bijux-dag-cli -- dag runs timeline first-hour --root runs --json
cargo run -p bijux-dag-cli -- dag runs verify first-hour --root runs --deep --json
```
