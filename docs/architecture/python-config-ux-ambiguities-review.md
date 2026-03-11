# Python Config UX Ambiguities Review

Scope: documented Python config UX ambiguities and parity constraints.

## Ambiguities Found

1. `config export --format text` is rejected even though text format exists globally.
2. `config export` response field `format: auto` does not explicitly reflect requested formatter.
3. `config load` behavior for missing source files is permissive in baseline flows and not obvious to users.
4. Error routing differences can include extra diagnostic lines in Python wrapper contexts.
5. `config reload` semantics are implicit: no explicit cache model is surfaced to users.

## Decision

Do not change these behaviors before parity freeze. Keep them documented and defer UX improvements to post-parity candidates.
