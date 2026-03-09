# Shell Completion Behavior

## Source of truth
- Root app construction in `src/bijux_cli/cli/root.py`
- REPL completion implementation in `src/bijux_cli/cli/repl/completion.py`
- REPL tests in `tests/unit/cli/commands/test_repl.py`

## Current behavior inventory
- No dedicated `completion` command is implemented as a built-in command group yet.
- Root Typer app uses framework-native completion behavior (install/show completion flags).
- REPL has explicit tab completion for:
  - built-ins (`exit`, `quit`)
  - top-level commands
  - subcommands and option flags
  - global flags (`--quiet`, `--format`, `--log-level`, `--pretty`, `--no-pretty`, `--help`)
  - `config set` placeholder completion (`KEY=VALUE`)

## Interaction behavior
- Tab cycles or starts completion.
- Enter accepts selected completion or executes line.
