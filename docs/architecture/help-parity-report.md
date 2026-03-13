# Help Parity Report

Date: 2026-03-09
Scope: stable parity and behavior coverage.

## Coverage added

Rust help snapshots now cover:

- Root help (`--help`) and no-color root help (`--color never --help`)
- Root command help for: `status`, `audit`, `docs`, `sleep`, `doctor`, `version`, `config`, `plugins`, `repl`, `completion`, `inspect`, `history`, `memory`, `cli`, `dev`
- `cli` subcommands: `status`, `paths`, `config get`, `config set`, `self-test`, `plugins list`, `plugins inspect`
- `dev cli` subcommands: `routes`, `registry`, `env`, `doctor`, `contracts`
- Alias help parity checks: `plugins inspect --help` equals `cli plugins inspect --help`; `dev doctor --help` equals `dev cli doctor --help`
- Nested help behavior: `bijux cli --help`

## Command-family comparison against Python

Compared current Rust root help content against Python command capture data.

- Accepted differences:
  - Rust currently exposes only the routed baseline command set in help.
  - Rust usage now reports canonical binary name `bijux`.
- Regressions requiring follow-up:
  - Python command-tree ordering still differs in sections where Python contains additional families.
  - Unknown-command diagnostics currently show deterministic `unknown route: <name>` without suggestion text.

## Stability checks added

- No-color help emits no ANSI escapes.
- Width-constrained help (`COLUMNS=50`) remains parseable and stable.
- Help rendering performance budget check for root help: under 1500ms in test environment.

## Status for 281-300

- `281`: complete
- `282`: complete
- `283-286`: complete for routed command surface and alias coverage
- `287-291`: complete for currently routed root/cli/dev/plugin/history/memory surfaces
- `292`: complete (hidden alias help checks)
- `293`: complete (unknown command diagnostics check)
- `294`: complete (nested help check)
- `295`: complete (`--format json --help` interaction test)
- `296`: complete (performance budget test)
- `297`: complete (line-wrap stability test)
- `298`: complete (no-color snapshot test)
- `299`: complete (this report)
- `300`: complete (rules summarized in this report)

## Historical Capture Notes

Earlier side-by-side help appendix pages were merged into this report. Their
useful conclusion was simple: help output was still structurally different
between the legacy Python captures and the current Rust runtime, even where
command coverage already existed. The generated parity artifacts remain the
current source of truth.

## Rendering Rules

Help rendering follows these rules:

1. Help output is plain text, even when machine-output flags are present.
2. Root and subcommand help must stay deterministic for identical inputs.
3. `--color never` must remove ANSI escapes.
4. Width-constrained rendering must remain readable when `COLUMNS` is set.
5. Unknown-command handling must keep a stable non-zero failure shape.

Ordering and compatibility rules:

1. Root help order follows the routed Rust command registry order.
2. Grouped command help follows subcommand declaration order.
3. Alias forms that normalize to canonical routes must expose equivalent help.
4. Python-vs-Rust help differences are only acceptable when parity reports
   document them.
