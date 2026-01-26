# Exit policy

Guarantees:

- Exit code is stable for a given error type
- Quiet suppresses output, not exit codes
- Output format does not change exit codes

User-visible effects:

- 0: success
- 2: usage or user input error
- 3: ASCII or encoding error
- 1: internal error
- 130: aborted by user
