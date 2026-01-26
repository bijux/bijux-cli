# Exit policy

The exit policy maps errors to exit codes and output routing.

Rules:

- Exit code is stable for a given error type
- Quiet suppresses output but does not change exit codes
- Output format never changes exit codes

Common codes:

- 0: success
- 2: usage or user input error
- 3: ASCII or invalid encoding
- 1: internal error
- 130: aborted by user

See reference/exit_codes.md for the canonical list.
