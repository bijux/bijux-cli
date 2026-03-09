# Config Get Post-Parity Improvements

This list captures safe improvements after baseline parity lock.

1. Add optional `--with-source` metadata mode (`source`, `source_path`, precedence trace).
2. Expose structured not-found diagnostics with suggestion hints for close keys.
3. Add optional typed coercion mode for common value shapes (`bool`, `int`, `duration`).
4. Add config-key schema discovery metadata for completion and docs integration.
5. Add cache-aware fast path for repeated reads in long-lived REPL sessions.
6. Add targeted benchmark suite with regressions tracked over representative config sizes.

None of these changes should be merged before explicit parity baseline decision updates.
