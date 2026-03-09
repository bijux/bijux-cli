# REPL Commands and Shortcuts

## Source of truth
- `src/bijux_cli/cli/commands/repl.py`
- `src/bijux_cli/cli/repl/ui.py`
- `src/bijux_cli/cli/repl/execution.py`
- `src/bijux_cli/cli/repl/parsing.py`

## Command behavior inventory
- Starts interactive session when launched without piped input and not in quiet mode.
- Runs piped mode when stdin is not a TTY or `--quiet` is set.
- Splits chained commands by `;` outside quoted segments.
- Ignores blank lines and comment-prefixed input.

## Built-in REPL shortcuts
- `exit`
- `quit`
- `docs`
- `docs <topic>`

## Key interaction shortcuts
- `Tab`: trigger/cycle completion.
- `Enter`: apply selected completion or submit command.
- `Ctrl+C`/`Ctrl+D` in interactive mode: clean exit.
- Signal handlers register clean exit for `SIGINT`, `SIGTERM`, `SIGHUP`, `SIGQUIT`, `SIGUSR1`.
