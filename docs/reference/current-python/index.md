# Current Python Behavior Inventory

## Purpose

Capture the historical Python runtime behavior that still matters for parity
work, migration review, and regression analysis.

## Scope

This section records behavior that was previously owned by the Python runtime
lineage. The current runtime is Rust-backed, so this inventory stays focused on
observable command behavior, snapshots, and compatibility notes rather than old
implementation paths.

## Command Inventory

Top-level commands covered by the archived Python behavior captures:

- `atlas`
- `audit`
- `config`
- `dev`
- `docs`
- `doctor`
- `help`
- `history`
- `memory`
- `plugins`
- `repl`
- `sleep`
- `status`
- `version`

Developer command behavior retained from the Python runtime:

- `bijux dev`
- `bijux dev <tool>`
- `bijux dev di`
- `bijux dev list-products`
- `bijux dev list-plugins`

Known delegated tool namespaces captured in that lineage:

- `agent`
- `atlas`
- `dag`
- `dna`
- `gnss`
- `rag`
- `rar`
- `vex`

## REPL And Completion Behavior

Recorded REPL behavior includes:

- interactive startup when stdin is a TTY
- piped mode when stdin is not a TTY or `--quiet` is set
- `;` splitting outside quoted segments
- blank-line and comment skipping
- built-in `exit`, `quit`, and `docs` shortcuts
- tab completion for commands, subcommands, flags, and selected placeholders

The historical Python command surface did not provide a dedicated `completion`
command group. Shell completion came from the framework-level install/show
flags, while interactive completion lived inside the REPL.

## Exit Codes Used In Tests

Observed exit codes retained in tests and captures:

- `0` success
- `1` internal or general failure
- `2` usage or user-input failure
- `3` serialization or encoding failure
- `130` user abort or signal interruption

Some unit coverage also used targeted non-contract values when testing explicit
passthrough behavior.

## Detailed Inventory Files
- [Global flags and precedence](global-flags-and-precedence.md)
- [Config paths and environment variables](config-paths-and-environment-variables.md)
- [Built-in command output shapes](builtin-command-output-shapes.md)
- [Error message and exception classes](error-message-and-exception-classes.md)
- [Plugin command and lifecycle behavior](plugin-command-and-lifecycle-behavior.md)
- [Golden outputs and behavior captures](golden-and-behavior-captures.md)
