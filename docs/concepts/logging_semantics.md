# Logging semantics

Log level controls verbosity and internal detail.

Levels:

- trace: most verbose, internal details
- debug: developer diagnostics
- info: default
- warn/error: only warnings and errors

Semantics:

- Quiet suppresses output, not exit codes
- Log level can be set by CLI flag or env var
- Output format does not affect logging decisions
