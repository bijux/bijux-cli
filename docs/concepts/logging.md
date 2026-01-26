# Logging

Semantics:

- trace: most verbose, internal details
- debug: diagnostics
- info: default
- warn/error: only warnings and errors

Rules:

- Quiet suppresses output, not exit codes
- Log level is resolved via precedence
