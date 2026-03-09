# Parity Release Candidate Rule

Parity reports are mandatory for every release candidate.

Required artifacts:
- `artifacts/parity/command_parity_matrix.json`
- `artifacts/parity/command_parity_diffs.json`
- `artifacts/parity/stdout_diff.md`
- `artifacts/parity/stderr_diff.md`
- `artifacts/parity/exit_code_diff.md`
- `artifacts/parity/help_diff.md`

Gates:
- CI fails if a parity-covered command regresses.
- CI prints warnings if parity-partial commands drift further away.

This rule is frozen until parity governance is explicitly replaced.
