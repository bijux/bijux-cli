# CLI usage

This guide covers real usage patterns and flags.

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

## Config workflow

```bash
bijux config set foo bar
bijux config get foo
bijux config list --format json
bijux config unset foo
```

## Plugins

```bash
bijux plugin list
bijux plugin info example
```
