# CLI Surface Stability Policy

## Scope

This policy governs the public `bijux dag` command surface, including flags, exit codes, JSON envelopes, and human-readable output expectations.

## Stability Rules

1. Existing top-level commands and documented aliases are stable by default.
2. JSON output envelopes must keep `command`, `status`, and `data` fields stable.
3. Exit code semantics must remain aligned with `configs/policy/error_codes.json`.
4. Help output may evolve wording, but command/flag presence must remain backward-compatible for stable surfaces.
5. Deprecated surfaces require documented migration guidance before removal.

## Change Requirements

- Any CLI-breaking change requires:
  - updated docs in `docs/spec`
  - regression tests in `crates/bijux-dag-cli/tests`
  - release note entry

## Non-Goals

- Freezing all help-text phrasing byte-for-byte
- Guaranteeing unchanged stderr wording for internal errors
