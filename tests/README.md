# Test placement guide

Use this guide to choose where tests belong.

## Crate tests (`crates/*/src/*` with `#[cfg(test)]`)

- fast unit and module behavior for one crate
- no process spawning for product binary

## Integration tests (`crates/*/tests/*.rs`)

- crate public API contracts
- command orchestration behavior at crate boundary
- error-path behavior for public surfaces

## End-to-end tests (`tests/e2e/*`)

- only place where tests may shell out to production binaries
- validate full workflows across crates and filesystem artifacts
- consume canonical scenario assets from `evidence/` rather than owning scenario JSON

## Taxonomy naming

Test file names must include one category prefix:

- `unit_`
- `contract_`
- `integration_`
- `e2e_`
- `perf_`
- `compat_`
- `fault_`
