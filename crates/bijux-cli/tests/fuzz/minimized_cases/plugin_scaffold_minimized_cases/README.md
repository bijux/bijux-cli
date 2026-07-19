# Plugin Scaffold Minimized Cases

These files preserve plugin scaffold command regressions. The replay in
`tests/integration/cli/plugins/plugin_scaffold_case_replays.rs` executes each
argument vector twice against an isolated plugin directory and requires the
exit status to remain deterministic.

## File Grammar

- one argument token per non-empty line
- lines beginning with `#` are comments
- `{ROOT}` expands to the replay suite's scratch workspace
- no shell quoting or variable expansion is performed

The retained cases cover successful Python and Rust scaffolds, a reserved
plugin name, and a parent-path traversal attempt.

## Replay

```sh
cargo test -p bijux-cli --test integration minimized_scaffold_cases_replay_with_deterministic_exit_codes
```

## Scope And Updates

This suite proves repeatable exit classification for retained argument vectors.
It does not inspect every generated file or prove scaffold content parity.
Content and lifecycle expectations belong in the plugin scaffold integration
tests.

Minimize new cases to the arguments required to reproduce the defect. Use
`{ROOT}` for writable paths, never a developer-specific absolute path. Keep
generated projects and reduction logs under `artifacts/`.
