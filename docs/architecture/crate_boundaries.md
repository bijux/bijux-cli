# Crate Boundaries

This document records current crate-boundary metrics and explicit boundary decisions.

## Source Metrics

Generated artifact:
- `artifacts/status/crate_boundary_metrics.json`

Metrics included:
- compile time per crate
- test build time per crate
- dependency fan-in per crate
- dependency fan-out per crate
- public API size per crate
- churn over recent commits
- cross-crate change frequency for key crate pairs

## Boundary Decisions

### `install` relative to `core`
- Decision: keep separate for now
- Status: watch
- Rationale: install contains path resolution and environment/package diagnostics that are operationally distinct from command execution semantics.

### `output` relative to `core`
- Decision: keep separate for now
- Status: watch
- Rationale: output formatting and machine-envelope rendering remain separable concerns with independent test coverage.

### `routing` relative to `core`
- Decision: keep separate for now
- Status: watch
- Rationale: parsing and route identity logic still benefit from dedicated fixture/property tests and independent drift checks.

### `python`
- Decision: must stay separate
- Status: keep
- Rationale: bridge and packaging behavior are language/runtime-boundary concerns.

### `contracts`
- Decision: must stay separate
- Status: keep
- Rationale: machine-readable contract stability must be independent of execution implementation details.

### `plugin`
- Decision: must stay separate
- Status: keep
- Rationale: plugin lifecycle and registry law are subsystem boundaries, not core execution primitives.

### `repl`
- Decision: must stay separate
- Status: keep
- Rationale: interactive session behavior and transcript parity are distinct runtime surfaces.

## No-Large-Merge Rule

Large crate merges are frozen until parity is stronger.

Definition used here:
- A large merge is any change that collapses an existing top-level workspace crate into another crate or introduces broad cross-crate module relocation.

Freeze condition:
- no large crate merge is allowed while parity coverage remains partial and mismatch reports are still active.

Unfreeze criteria:
- parity matrix coverage is broad and stable,
- mismatch trend is consistently improving,
- boundary costs exceed separation benefits per updated metrics.
