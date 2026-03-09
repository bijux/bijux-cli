# Global Flags and Precedence

## Source of truth
- `src/bijux_cli/cli/core/constants.py`
- `src/bijux_cli/cli/core/flags.py`
- `src/bijux_cli/core/intent.py`
- `src/bijux_cli/core/precedence.py`

## Global flags
- `-q`, `--quiet`
- `-f`, `--format`
- `--log-level`
- `--color`
- `--pretty`, `--no-pretty`
- `-h`, `--help`

## Parsing and validation behavior
- Flag parsing is tolerant and collects structured validation errors.
- `--help` short-circuits parse-error collection.
- `--format` accepted values in structured mode: `json`, `yaml`.
- REPL command validates and only accepts `human` for REPL session format.

## Effective precedence behavior
- Current implementation precedence for intent flags is: CLI flags -> environment -> defaults.
- `quiet=true` forces effective log level to `error`.
- Color resolution also applies `NO_COLOR=1` override in intent color resolution.
