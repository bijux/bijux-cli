# Config Minimized Cases

These `.env` payloads preserve configuration parser and command regressions.
`tests/integration/cli/config/config_case_replays.rs` copies each file to a
scratch config path, then runs `config list` and `config load` twice.

The replay requires stable exit status, stdout, and stderr across both runs. It
also enforces the CLI stream contract: success has stdout and no stderr;
failure has stderr and no stdout.

## Retained Behaviors

- a malformed line after a valid assignment
- duplicate keys
- quoted and escaped values
- an embedded null byte

## Replay

```sh
cargo test -p bijux-cli --test integration minimized_config_cases_replay_with_stable_exit_behavior
```

## Updating The Corpus

Store the raw bytes expected in a user config file; do not encode them as JSON
or normalize invalid bytes. Reduce a new case until removing any remaining
line or byte would stop reproducing the behavior. The replay checks
determinism, not whether a particular payload should succeed, so a behavior
change also needs an assertion in the owning config contract test.

Temporary files, fuzzer output, and unreduced candidates belong under
`artifacts/`.
