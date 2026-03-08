# Operator workflows

Audience: operators  
Owner: operator experience maintainers  
Status: stable

Use this page for day-to-day run investigation and integrity workflows.
For full command taxonomy and compatibility policy, use `docs/reference/COMMAND_TAXONOMY.md`.

## Common tasks

### Check recent run health

```sh
cargo run -p bijux-dag-cli -- dag runs list --root runs
cargo run -p bijux-dag-cli -- dag runs show <run_id> --root runs
```

### Investigate failures

```sh
cargo run -p bijux-dag-cli -- dag runs inspect <run_id> --root runs
cargo run -p bijux-dag-cli -- dag runs timeline <run_id> --root runs
cargo run -p bijux-dag-cli -- dag runs explain-failure <run_id> --root runs
```

### Compare and replay decisions

```sh
cargo run -p bijux-dag-cli -- dag runs diff <run_a_dir> <run_b_dir> --explain
cargo run -p bijux-dag-cli -- dag runs compare <run_a> <run_b> --root runs
```

### Validate run-directory integrity

```sh
cargo run -p bijux-dag-cli -- dag runs verify <run_id> --root runs --deep --json
cargo run -p bijux-dag-cli -- dag runs doctor <run_id> --root runs
```

## Investigation order

1. `dag runs show`
2. `dag runs inspect`
3. `dag runs timeline` and `dag runs tree`
4. `dag runs explain-failure`
5. `dag runs verify` and `dag runs doctor`
