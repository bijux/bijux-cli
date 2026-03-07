# Benchmarks Contract

## Scope
`benchmarks/` stores benchmark scenarios, baseline formats, and benchmark evidence artifacts.

## Authority
This directory is authoritative for benchmark scenario definitions and baseline comparison structure.

## Invariants
- Micro and system benchmarks are separated.
- Baseline format is versioned and machine-readable.
- Performance claims require evidence artifacts under benchmark outputs or baselines.

## Allowed changes
- Add scenarios with workload metadata.
- Add structured evidence fields without breaking existing readers.

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs` benchmark compare and performance claim guard
- `crates/bijux-dag-core/benches/micro_parse_validate.rs`

## Related schemas
- `benchmarks/baselines/benchmark_report.schema.json`

## Versioning and change policy
Baseline schema changes require a new schema version and migration notes.
