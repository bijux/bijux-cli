# REPL Library Audit

## Scope

This audit covers `crates/bijux-cli-repl/src/lib.rs` as of commit `a819a12` and classifies major behavior surfaces before modularization.

## Findings

1. Session lifecycle, history persistence, completion, execution, diagnostics, and reference rendering were all implemented in a single file.
2. Public API shape was broad but coherent; function boundaries were stable enough to preserve as re-exports.
3. Runtime command flow already delegated parse/routing/kernel/emission to shared crates, which is aligned with parity goals.
4. History behavior used JSON persistence with tolerant malformed-file recovery semantics.
5. Transcript behavior included interrupt and EOF handling, plus meta-commands (`:help`, `:set`, `:plugin reload`, `:exit`).

## Classification by area

- Session bootstrap/shutdown: parity-partial (stable scaffolding, limited session identity strategy)
- History load/flush/replay: parity-partial (covered for cap and corruption handling; migration compatibility needed)
- Completion hooks: parity-partial (built-ins + plugin namespaces; reserved namespace behavior needed)
- Execution pipeline: parity-partial (shared parser/router/kernel path in place; deeper transcript parity still needed)
- Diagnostics and budgets: placeholder-compatible (useful guards, not Python parity surface)
- Reference rendering: compatibility shim (stable local reference text, not Python-generated)

## Modular ownership after split

- `session.rs`: startup/shutdown/marker/policy
- `history.rs`: configure/load/flush/replay
- `completion.rs`: completion candidates + plugin hooks
- `execution.rs`: line/input execution and meta command flow
- `diagnostics.rs`: session diagnostics and budget checks
- `reference.rs`: command reference rendering
- `types.rs`: shared contracts and error types

## Follow-up focus

1. Expand transcript parity suite with explicit help/plugin/error/quiet/json/yaml/interrupt/EOF cases.
2. Add parser parity assertions between REPL token flow and CLI parser intent.
3. Add Python history layout migration fixtures.
4. Publish a REPL parity status report artifact.
