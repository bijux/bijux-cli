# dev-dag Command Module Boundaries

## Dispatch ownership
- `crates/bijux-dev-dag/src/commands/mod.rs` keeps top-level CLI dispatch and cross-suite orchestration.

## Extracted boundaries
- Performance evidence routing and reports: `commands/perf_evidence.rs`
- Suite catalog rendering and registry surfaces: `commands/suite_catalog.rs`
- Report writing helpers: `report/write.rs`
- Cargo tooling wrappers: `tooling/cargo.rs`
- Git tooling wrappers: `tooling/git.rs`
- Shared command runner interface: `tooling/mod.rs`

## Runtime helper extraction
- Process execution helpers used by command dispatch: `commands/command_runtime.rs`
