# Dev control command responsibility inventory

## Command ownership in `commands/mod.rs`

The command router owns only:

- CLI argument parsing and command match routing
- choosing command group handlers
- composing stable command IDs for reports

The command router does not own:

- report serialization and audit append logic
- suite selection and filtering policy
- suite explanation payload generation

## Extracted authority modules

- `commands/model.rs`
  - typed command effect model
  - typed suite definition model
  - typed suite selection report model
- `commands/reporting.rs`
  - command report emission
  - report file writing
  - audit append record
- `commands/suite_dispatch.rs`
  - suite filtering with domain/slow/internal/override policy
  - suite explain/list output payloads
  - advisory vs blocking suite result semantics

## Current decomposition gaps

The command router still contains many command handlers with direct operational logic. The next split should move command families into dedicated domain modules and keep `commands/mod.rs` as wiring and dispatch only.
