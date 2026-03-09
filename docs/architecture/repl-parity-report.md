# REPL Parity Report

Date: 2026-03-09

## Inputs used for comparison

Python references:

- `src/bijux_cli/cli/repl/parsing.py`
- `src/bijux_cli/cli/repl/execution.py`
- `src/bijux_cli/cli/repl/ui.py`
- `artifacts/python-behavior/runtime/repl-interactive.txt`

Rust references:

- `crates/bijux-cli-repl/src/session.rs`
- `crates/bijux-cli-repl/src/history.rs`
- `crates/bijux-cli-repl/src/completion.rs`
- `crates/bijux-cli-repl/src/execution.rs`
- `crates/bijux-cli-repl/tests/transcript_parity.rs`
- `crates/bijux-cli-repl/tests/transcript_cases.rs`

## Parity status

| Behavior | Status | Evidence |
|---|---|---|
| Session startup/shutdown flow | Partial parity | `startup_repl` and `shutdown_repl` tests |
| Help transcript | Parity-covered | `transcript_case_help_command` |
| Plugin command transcript | Parity-covered | `transcript_case_plugin_command` |
| Error transcript | Parity-covered | `transcript_case_error_command` |
| Quiet transcript behavior | Parity-covered | `transcript_case_quiet_mode` |
| JSON transcript behavior | Parity-covered | `transcript_case_json_mode` |
| YAML transcript behavior | Parity-covered | `transcript_case_yaml_mode` |
| Interrupt behavior | Parity-covered | `transcript_case_interrupt` |
| EOF exit behavior | Parity-covered | `transcript_case_eof_exit` |
| History malformed-file recovery | Parity-covered | `malformed_history_recovers_without_crashing` |
| History large-file handling | Parity-covered | `large_history_load_stays_within_sanity_budget` |
| Python history layout compatibility | Parity-covered | `history_file_supports_python_prompt_toolkit_layout` |
| Parser parity with CLI parser | Parity-covered | `repl_line_tokenization_matches_cli_parser_expectations` |
| Completion reserved namespaces | Parity-covered | `completion_includes_reserved_namespace_candidates` |
| Completion plugin namespaces | Parity-covered | `completion_includes_plugin_namespace_candidates` |
| Startup without config/registry | Parity-covered | `startup_works_without_config_or_plugin_registry` |

## Explicit mismatches and gaps

1. Python interactive UI uses prompt-toolkit and colored prompt rendering; Rust REPL crate currently validates runtime behavior but does not duplicate prompt-toolkit UX.
2. Python supports semicolon command splitting and certain piped-mode affordances that are not fully replicated in Rust tests.
3. Python emits some no-command/unknown-command wording from UI helpers; Rust wording for equivalent errors is stable but not string-identical.

## Next steps

1. Add semicolon-segment transcript parity fixtures from Python REPL scripts.
2. Add explicit piped-mode transcript fixtures for line comments and doc shortcuts.
3. Decide whether prompt-toolkit-specific UX is required for parity scope or intentionally excluded.
