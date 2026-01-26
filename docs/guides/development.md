# Development

Use this guide to navigate the repo, run gates, and keep outputs clean.

## Repository layout

- `src/`: production code
- `tests/`: unit, regression, e2e, nightly, benchmark
- `docs/`: authored documentation
- `config/`: tool configuration
- `makefiles/`: single source of truth for tooling
- `scripts/`: build helpers and CI tooling
- `artifacts/`: generated output only

## Core commands

```bash
make lint
make quality
make security
make test
make test-all
make api
```

## Toolchain

The toolchain is consistent in CI and locally.

- Formatting and linting: Ruff
- Typing: Mypy
- Docstring style: Pydocstyle (Google)
- Doc coverage: Interrogate
- Complexity: Radon
- Dead code: Vulture
- Dependency hygiene: Deptry
- License compliance: REUSE
- Security: Bandit and pip-audit

## Tests

Test layers are explicit and marker-driven.

- Unit: `tests/unit`
- Regression: `tests/regression`
- E2E: `tests/e2e`
- Nightly: `tests/nightly`
- Benchmark: `tests/benchmark`

Run layer-specific targets via the Makefile. Use markers for selection.

## Docs workflow

Docs are written under `docs/` and built into `artifacts/docs`.
All documentation follows a single-heading style and consistent spacing.

## Docstrings

Docstrings follow the Google style guide.

- Each module starts with a module-level docstring.
- Functions and classes have explicit Args/Returns/Raises where applicable.
- Pydocstyle enforces conformance in CI.

## Artifacts only

Generated outputs must go under `artifacts/` and nowhere else.
See `architecture/decision-rules.md` for the enforcement policy.

## Log level refactor inventory

Scope: replace boolean debug toggles with `LogLevel` comparisons.

- CLI options and config resolution
- Diagnostics and emitter helpers
- Observability and telemetry logging thresholds
- Tests and docs referencing `--log-level debug`

Replacement rule: treat `LogLevel.DEBUG` or lower as the diagnostics threshold.
