# Get started

This guide gets you from zero to first command in minutes.

## Install

Use pip:

```bash
pip install bijux-cli
```

Or hatch (dev environments):

```bash
hatch shell
pip install -e .
```

## Verify install

```bash
bijux --help
bijux version
```

Expected output conventions:

- JSON/YAML output for structured commands
- Human help text for help output
- Exit codes follow the exit policy rules (see reference/exit_codes.md)

## First commands

```bash
bijux status
bijux config set foo bar
bijux config get foo
bijux config unset foo
```

## Next steps

- Read concepts/precedence.md
- Try guides/cli_usage.md
- Explore examples/basic_example.md
