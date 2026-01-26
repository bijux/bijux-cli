# Precedence

## Purpose
This document guarantees how configuration and flags override each other.

## Scope
This covers CLI flags, environment, config files, and defaults only.

## Core Concepts
- Higher layers override lower layers.
- Flag precedence is explicit and short-circuiting.

## Invariants
- CLI flags always override environment values.
- Environment values always override config values.
- Config values always override defaults.
- Output format does not change precedence.

## Execution
### Resolution order
1. CLI flags
2. Environment
3. Config file
4. Defaults

### Flag precedence
1. `--help`: short-circuit, exit 0.
2. `--quiet`: suppress output, keep exit code.
3. `--log-level debug`: diagnostics, forces pretty output.
4. `--format json|yaml`: structured output, invalid value exits 2.
5. `--pretty` or `--no-pretty`: indentation only.

## Failure Modes
- Invalid format: exit code 2 with structured error.
- Invalid flag value: exit code 2.
- Non-ASCII inputs in config or env: exit code 3.

No recovery occurs inside the CLI. The process exits deterministically.

## Design Rationale
- Alternatives: per-command precedence or late overrides.
- Rejected because it creates drift and hidden behavior.
- Chosen: single precedence policy for all commands.

## Non-Goals
- Command-specific behavior.
- Plugin-defined flags.

## References
- Implementation: `src/bijux_cli/core/precedence.py`
- Regression coverage: `tests/regression/test_precedence.py`
