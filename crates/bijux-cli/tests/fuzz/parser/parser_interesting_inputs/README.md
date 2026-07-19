# Parser Interesting Input Corpus

This corpus exercises combinations that are valid inputs to parser exploration
but are not necessarily minimized regressions. The replay in
`tests/routing/parser/parser_case_replays.rs` parses every input and, when a
normalized route is present, asks the default registry to resolve it.

The files currently cover built-in and external namespaces, global flag
ordering, conflicting output flags, root help, unknown flags, misspellings, and
Unicode confusables.

## Input Grammar

Each non-empty, non-comment line is one whitespace-delimited argument vector.
The parser does not interpret shell quotes in this corpus. Inputs normally
start with `bijux` because `parse_intent` receives the executable token.

## Replay

```sh
cargo test -p bijux-cli --test routing interesting_corpus_cases_do_not_crash_or_corrupt_route_resolution
```

## Admission Rule

Add a case when it represents a distinct parser interaction worth preserving
across the whole input space. A specific regression should be reduced and
stored in `../parser_minimized_cases/` instead. The replay guarantees parser
acceptance without a crash and safe route lookup; it does not assert a
particular command result for every line.
