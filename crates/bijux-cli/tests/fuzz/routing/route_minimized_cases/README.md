# Route Minimized Cases

These files retain namespace sets that previously exposed route registration
and normalization risks. The replay in
`tests/routing/laws/route_case_replays.rs` registers each set in forward and
reverse order and compares both the route tree and rendered command tree.

## Retained Behaviors

- normalized collisions across hyphens, underscores, and case
- external namespaces that overlap built-in command roots
- mixed names whose registration order must not affect output

Each non-empty, non-comment line is one namespace token. One file represents
one registration set.

## Replay

```sh
cargo test -p bijux-cli --test routing minimized_route_cases_do_not_crash_and_are_deterministic
```

## Updating The Corpus

Reduce a case to the smallest namespace set that still demonstrates
order-dependent or normalization-sensitive behavior. Preserve the meaningful
ordering in the file even though the test also reverses it. This suite checks
deterministic trees; separate routing contracts must assert whether a
particular collision is accepted or rejected.
