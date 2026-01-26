# Precedence

Guarantee: higher layers override lower layers.

## Resolution order

1. CLI flags
2. Environment
3. Config file
4. Defaults

## Flag precedence (global)

1. `--help`: short-circuits, exit 0
2. `--quiet`: suppress output, preserve exit code
3. `--log-level debug`: diagnostics, forces pretty output
4. `--format json|yaml`: structured output, invalid value exits 2
5. `--pretty` / `--no-pretty`: indentation only

## Invariants

- Explicit inputs always win
- Defaults never override explicit inputs
- Output format never changes precedence
