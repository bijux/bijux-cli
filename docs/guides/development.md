# Development

## Purpose
This document tells you how to run developer workflows and gates.

## Scope
It covers repository layout and tooling, not product behavior.

## What problem this solves
If local runs diverge from CI, bugs slip in.
This guide keeps local and CI commands identical.

## Why you should care
If you use the same gates as CI, your changes pass on the first try.

## What confusion this removes
It removes doubt about where outputs go and which command to run.

## Guarantees
Bijux guarantees:
1. CI runs the same Make targets as local workflows.
2. Generated output never lands in the repo root.

## How to Think About This
Treat Make targets as the API for developer work.

## Common Misunderstandings
- "I can run tools directly." You can, but CI uses Make targets.

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
We deliberately chose Make targets to keep CI and local in sync.
Why not per-tool scripts? They drift and hide missing steps.

## Non-Goals
- OS-specific package management.

## References
- Decision rules: `architecture/decision-rules.md`
