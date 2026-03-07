# CI Integration

## Minimal CI usage
```sh
cargo run -p bijux-dag-cli -- dag validate dag.json --strict
cargo run -p bijux-dag-cli -- dag run dag.json --out runs --run-id ci-run --json
cargo run -p bijux-dag-cli -- dag runs verify ci-run --root runs --deep --json
```

## Recommended governance checks
```sh
cargo run -p bijux-dev-dag -- repo run --domain governance
```
