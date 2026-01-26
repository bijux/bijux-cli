# CLI usage

Use this guide to run commands with stable, scriptable output.

## Platform support

Supported: Linux and macOS.
Not supported: Windows (POSIX locks and filesystem behavior required).

## Quick commands

```bash
bijux --help
bijux --version
bijux doctor
```

## Output formats

```bash
bijux status --format json
bijux status --format yaml
```

## Quiet mode

```bash
bijux status --quiet
```

## Log level

```bash
bijux status --log-level debug
```

## Shell completion

```bash
bijux --install-completion
bijux --show-completion
```

## Global precedence

Global flags resolve in strict order. See `concepts/precedence.md`.

## Command reference

Full command list and flags live in `reference/commands.md`.

## Errors and exit codes

Structured errors follow the selected output format unless `--quiet`.
Exit codes are listed in `reference/exit-codes.md`.
