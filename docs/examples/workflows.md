# Workflow Examples

## Purpose
These examples show how bijux-cli behaves in realistic workflows. Each scenario focuses on a single outcome so you can see how configuration, output formatting, and exit behavior combine in practice.

## Scope
The examples here use core commands only. They do not require plugins or custom configuration files unless explicitly stated.

## Example 1: Script‑Safe Status Check
This example demonstrates how to obtain machine‑readable output for automation.

### Setup
No configuration changes are required.

### Command
```bash
bijux status --format json
```

### Expected Output
A JSON payload with a top‑level status value. Exact keys may evolve, but the payload is valid JSON and the exit code is 0 on success.

### Why This Matters
Explicit JSON output makes the command safe for CI and scripts. It also guarantees that terminal styling will not interfere with parsing.

## Example 2: Deterministic Precedence Resolution
This example shows how CLI flags override environment values and config files.

### Setup
Set an environment variable and a config value for the same key.

```bash
bijux config set foo=from_config
export BIJUXCLI_FOO=from_env
```

### Command
```bash
bijux config get foo
```

### Expected Output
The output should reflect the environment value, because env overrides config. If you pass a CLI flag that sets the same value, the CLI value wins.

### Why This Matters
Understanding precedence prevents confusion when multiple sources define the same key. It also keeps automation predictable when environments differ.

## Example 3: Exit Policy in Failure Conditions
This example demonstrates stable exit codes for invalid inputs.

### Setup
None.

### Command
```bash
bijux config get missing_key
```

### Expected Output
The command fails with a structured error message and a stable non‑zero exit code. Scripts should rely on the exit code rather than parsing the error text.

### Why This Matters
Stable exit codes are the CLI’s contract with automation. They provide a reliable failure signal even when output formats change.
