# Configuration

## Purpose
This guide explains how to configure bijux-cli safely and predictably. It exists to help you set values in a way that respects precedence and avoids hidden overrides.

## Scope
It covers configuration sources, how to set values, and how to validate results. It does not describe all possible configuration keys; use the reference for that.

## Configuration Sources
Bijux reads configuration from CLI flags, environment variables, and configuration files. The precedence rules determine which value is active when multiple sources define the same key.

## Setting a Value
Use `bijux config set` with a `KEY=VALUE` pair to define a setting explicitly.

```bash
bijux config set foo=bar
```

This updates the configuration file and makes the value visible to subsequent commands unless overridden by a higher-precedence source.

## Verifying Configuration
Use `bijux config get` to check the active value. This lets you confirm that precedence produced the expected result.

```bash
bijux config get foo
```

## Why This Matters
Configuration mistakes are a common source of confusion in automation. Explicit commands and clear precedence rules reduce this risk.

## References
- [Precedence](../concepts/precedence.md)
- [Config schema](../reference/config-schema.md)
