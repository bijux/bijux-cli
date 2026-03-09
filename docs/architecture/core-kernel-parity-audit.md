# Core Kernel Parity Audit

Date: 2026-03-09
File: `crates/bijux-cli-core/src/kernel.rs`

## Path Classification

### Parity-complete
- Flag precedence resolution in `resolve_policy` (`flags > env > config > defaults`).
- Quiet-mode log-level override behavior (`quiet -> log_level=error`).
- Unified sync/async handler pipeline entry (`execute_pipeline`).
- Stable category-to-exit mapping helper (`map_error_category_to_exit`).

### Parity-partial
- Fast-path behavior (`help|version|completion`) returns generic placeholder payload instead of command-specific parity payloads.
- Trace payload includes stable structure but fixed timestamps and IDs.
- Lifecycle hook trigger points exist, but route coverage is partial.

### Stub / placeholder
- `build_intent_from_argv` is a lightweight parser and not a full parity parser.
- `error_envelope` source-category mapping uses coarse placeholders.
- Timeout/cancel handling in tests covers mechanics, not full user-facing parity semantics.

## Required Follow-up For Full Parity
- Route kernel intent from routing parser output rather than lightweight parser path extraction.
- Replace fast-path placeholder payload with command-specific parity payload builders.
- Align trace event metadata with final runtime diagnostics contract.
