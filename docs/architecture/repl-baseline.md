# REPL Baseline

This document freezes the first Rust REPL baseline.

## Baseline behavior

1. REPL uses the same core app execution path for normal commands.
2. REPL output format, quiet mode, and trace mode are controlled via `:set` commands.
3. REPL interrupt and EOF behavior are deterministic and covered by transcript tests.
4. REPL history loading is resilient to malformed files and bounded by configured limits.
5. REPL completion includes root, grouped namespace, plugin namespace, and partial token coverage.

## Baseline parity rules

1. For covered commands, REPL output payloads must match non-interactive CLI payloads for equivalent policy flags.
2. Usage failures return stable diagnostics while keeping session continuity.
3. Stream routing follows core behavior (`stdout` success, `stderr` failures).

## Baseline exclusions

1. Python prompt-toolkit rendering details.
2. Semicolon command chaining semantics.
3. Extended piped-mode REPL shortcuts.

## Change control

Any REPL behavior change must include:

1. Transcript test updates.
2. Parity report update.
3. Explicit compatibility decision if non-interactive parity is impacted.
