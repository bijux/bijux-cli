# Runtime module ownership report

## Current inventory

- runtime rust module files (`crates/bijux-dag-runtime/src/*.rs`): `84`
- sacred module inventory maintained in `docs/architecture/runtime_module_triage.md`

## Ownership model

- core execution and scheduling: runtime maintainers
- artifact interaction boundaries: runtime + artifact maintainers
- governance and policy validation: control-plane maintainers

## Reduction note

- baseline triage inventory is tracked in runtime module triage doc
- reduction objective remains active until non-core support modules are further collapsed
