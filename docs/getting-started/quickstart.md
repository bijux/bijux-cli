# Quickstart

Goal: run a full config cycle.

```bash
bijux config set foo=bar
bijux config get foo
bijux config list --format json
bijux config unset foo
```

Expected:

- JSON/YAML output for structured commands
- Exit codes are stable (see reference/exit-codes.md)
