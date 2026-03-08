# CLI ownership boundaries

`bijux-cli` owns entrypoint composition and shell-level command wiring.

`bijux-dag` owns DAG command semantics, output envelopes, exit behavior, and compatibility guarantees.

## Responsibility split

- `crates/bijux-dag-cli`:
  - top-level `bijux` command tree
  - sub-app mounting (`dag`)
  - completions surface
- `crates/bijux-dag-app`:
  - `dag` command behavior and routing
  - JSON and text response contracts
  - legacy alias behavior (`dag status`, `dag verify`, `dag diff`)

## Change policy

- DAG semantics changes must land in `bijux-dag-app` with contract tests.
- `bijux-dag-cli` may not implement runtime semantics.
- Command taxonomy updates must update:
  - `docs/CLI_COMMAND_TAXONOMY.md`
  - `docs/CLI.md`
  - CLI contract tests in `crates/bijux-dag-cli/tests`.
