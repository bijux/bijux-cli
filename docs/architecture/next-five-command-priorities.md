# Next Five Command Priorities

Date: 2026-03-09

Selection basis: observed user value in current Python captures and parity impact.

1. `history` (read-only)
2. `plugins check <plugin>` (diagnostics)
3. `config` (config)
4. `plugins list` (plugin read path)
5. `repl --help` (REPL parity gap)

Current implementation status:

- `history`: implemented in Rust baseline routes.
- `plugins check <plugin>`: implemented in Rust baseline routes.
- `config`: implemented in Rust baseline routes.
- `plugins list`: implemented in Rust baseline routes.
- `repl --help`: covered by binary help path and parity runner command set.
