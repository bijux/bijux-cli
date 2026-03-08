# ADR: App Routing Post-Extraction End-State

## Decision

`crates/bijux-dag-app/src/lib.rs` is treated as dispatch and shared utility surface, while route modules own command-family behavior, response shaping, and text rendering entrypoints.

## Rationale

- keep command-family logic localized
- reduce router drift risk in the app crate
- keep ownership explicit for response/render/path/precondition helpers

## Guardrails

- architecture drift tests enforce route ownership boundaries
- generated route responsibility/coupling/import reports are required release artifacts
