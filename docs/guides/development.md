# Development

## Purpose
This document guarantees how to run developer workflows and gates.

## Scope
It covers repository layout and tooling, not product behavior.

## Core Concepts
- Make targets are the single source of truth.
- Artifacts are written under `artifacts/` only.

## Invariants
- CI runs the same Make targets as local workflows.
- Generated output never lands in the repo root.

## Execution
Repository layout:

- `src/`: production code
- `tests/`: unit, regression, e2e, nightly, benchmark
- `docs/`: authored documentation
- `config/`: tool configuration
- `makefiles/`: tool orchestration
- `scripts/`: build helpers
- `artifacts/`: generated output only

Core commands:

```bash
make lint
make quality
make security
make test
make test-all
make api
```

## Failure Modes
- Missing toolchain dependencies cause Make target failures.
- Writing outside `artifacts/` is a build error.

## Design Rationale
- Alternatives: per-tool scripts.
- Rejected because they drift from CI.

## Non-Goals
- OS-specific package management.

## References
- Decision rules: `architecture/decision-rules.md`
