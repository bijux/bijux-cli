# Diagnostics Baseline

This document freezes the first Rust diagnostics baseline.

## Baseline command set

- `inspect`
- `cli inspect`
- `dev cli routes`
- `dev cli registry`
- `dev cli env`
- `dev cli doctor`
- `dev cli contracts`

## Baseline output rules

1. Machine-output mode defaults to JSON and remains deterministic for identical inputs.
2. Successful diagnostics payloads emit on stdout.
3. Usage/help failures emit normalized help text on stderr with non-zero exit.
4. Quiet mode suppresses output while preserving exit semantics.
5. Trace mode must not mutate functional diagnostics payload results.

## Baseline metadata guarantees

1. Route diagnostics include owner and source markers.
2. Registry diagnostics include ownership and precedence metadata.
3. Environment diagnostics include active path set and source precedence order.
4. Doctor diagnostics include grouped issues (`config`, `paths`, `plugins`).
5. Contract diagnostics include schema and version metadata.

## Change control

Any diagnostics output shape change must include:

1. Snapshot updates for affected commands.
2. Parity report updates.
3. Explicit compatibility decision when Python parity is impacted.
