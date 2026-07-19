# Minimized Parser Cases

These argument vectors are reduced parser regressions replayed by
`tests/routing/parser/parser_case_replays.rs`. Each case is parsed twice and
must produce identical command paths, normalized paths, and global flags.

The retained set covers repeated and conflicting presentation flags, an
external namespace route, an oversized token, and a Unicode-confusable help
token.

## Input Grammar

Each non-empty, non-comment line is one whitespace-delimited argument vector.
Shell quoting is not interpreted. Keep one reproducible vector per `.argv`
file so a failure identifies one input.

## Replay

```sh
cargo test -p bijux-cli --test routing minimized_parser_cases_do_not_crash_and_are_deterministic
```

## Updating The Corpus

Start from the failing vector and remove tokens or shorten values until any
further reduction loses the behavior. A retained case needs a corresponding
semantic assertion elsewhere when correctness means more than deterministic
parsing. Broad combinations that have not demonstrated a regression belong in
`../parser_interesting_inputs/`.

Fuzzer queues, crashes awaiting reduction, and run logs belong under
`artifacts/`.
