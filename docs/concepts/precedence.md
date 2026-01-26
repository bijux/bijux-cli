# Precedence

## Purpose
This document tells you exactly which input wins when multiple sources conflict.

## Scope
It covers CLI flags, environment variables, config files, and defaults.

## What problem this solves
Without a fixed order, the same command behaves differently across machines.
Precedence removes that ambiguity and protects automation.

## Why you should care
When you debug a bad value, you need to know where it came from.
Precedence gives you that answer in one place.

## What confusion this removes
It removes guesswork about which source overrides another.

## Guarantees
Bijux guarantees:
1. CLI flags override everything.
2. Environment overrides config files.
3. Config files override defaults.
4. Output format never changes precedence.

## How to Think About This
Think of each source as a stack of decisions. The top decision wins.
If you set a value twice, the top decision must override the lower one.

## Common Misunderstandings
- "Format changes precedence." It does not. Format only changes output rendering.
- "Defaults can override explicit input." They cannot.

## Execution
Resolution order:
1. CLI flags
2. Environment
3. Config file
4. Defaults

Flag order:
1. `--help`: short-circuit, exit 0.
2. `--quiet`: suppress output, keep exit code.
3. `--log-level debug`: diagnostics, forces pretty output.
4. `--format json|yaml`: structured output, invalid value exits 2.
5. `--pretty` or `--no-pretty`: indentation only.

## Failure Modes
- Invalid format: exit 2 with structured error.
- Invalid flag value: exit 2.
- Non-ASCII in config or env: exit 3.

No recovery happens inside the CLI. The process exits deterministically.

## Design Rationale
We deliberately chose a single precedence chain to prevent hidden overrides.
Why not allow per-command precedence? It breaks predictability for scripts.

## Non-Goals
- Command-specific precedence.
- Plugin-defined flags.

## References
- Implementation: `src/bijux_cli/core/precedence.py`
- Regression coverage: `tests/regression/test_precedence.py`
