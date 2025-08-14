# Comprehensive Testing in Bijux CLI
<a id="top"></a>

**Predictable quality at scale.** Bijux CLI ships with a deep, layered test suite (≈ **2,664 tests**) covering units, integrations, functional scenarios, and full **E2E** flows.  
**Current status:** **98%+ line coverage** (coverage.py), enforced in CI across Python 3.11–3.13.

[Back to top](#top)

---

## Table of Contents

- [Testing Philosophy](#testing-philosophy)
- [Suite Layout](#suite-layout)
- [Test Types](#test-types)
  - [Unit](#unit)
  - [Integration](#integration)
  - [Functional](#functional)
  - [End-to-End (E2E)](#end-to-end-e2e)
- [Fixtures & Data](#fixtures--data)
- [Conventions & Markers](#conventions--markers)
- [Running Tests](#running-tests)
- [Selecting/Scoping Tests](#selecting-scoping-tests)
- [Coverage & Quality Gates](#coverage--quality-gates)
- [CI Pipeline](#ci-pipeline)
- [Mutation & Property-Based Testing](#mutation--property-based-testing)
- [Performance, Flakes, Determinism](#performance-flakes-determinism)
- [Contributing New Tests](#contributing-new-tests)
- [Quick Commands Cheat Sheet](#quick-commands-cheat-sheet)
- [Links](#links)

[Back to top](#top)

---

<a id="testing-philosophy"></a>

## Testing Philosophy

We optimize for **predictability, depth, and maintainability**:

- **Determinism** — stable outputs, structured errors, fixed flag precedence.  
- **Layered depth** — unit → integration → functional → E2E, each with clear responsibility.  
- **Design pressure** — DI-friendly seams, pure serialization, thin CLIs.  
- **Speed** — fast unit feedback; heavier scenarios gated under markers.

[Back to top](#top)

---

<a id="suite-layout"></a>

## Suite Layout

Abridged structure under `tests/`:

```

tests/
unit/
commands/ core/ infra/ root/ services/
integration/
functional/
e2e/
api/ dev/ history/ plugins/ repl/  # + fixtures

```

> Shared fixtures live in `conftest.py` at the right level; E2E adds JSON/YAML “shape” fixtures for contract-like assertions.

[Back to top](#top)

---

<a id="test-types"></a>

## Test Types

<a id="unit"></a>

### Unit

**Goal:** isolate a single module/service.

- **Core:** DI graphs/resolution, engine loop, enums/exceptions.  
- **Infra:** serializer (JSON/YAML, redaction), retry/backoff with jitter, observability/telemetry, subprocess wrappers.  
- **Services:** config/history/memory/plugins/docs/doctor.  
- **Commands:** validation, parsing, stdout/stderr shaping.

Patterns: parametrization; mocks for I/O/clock/telemetry; assert side-effects (atomic history writes, locks).

[Back to top](#top)

---

<a id="integration"></a>

### Integration

**Goal:** verify **wiring** (service ↔ contract ↔ infra) through DI.

Focus: realistic pipelines, cross-layer config propagation, contention (history locks), error surfacing.

[Back to top](#top)

---

<a id="functional"></a>

### Functional

**Goal:** assert user-visible command behavior.

Focus: global flag precedence (help/quiet/debug/format/pretty/verbose), structured stdout/stderr, exit codes, non-interactive REPL.

[Back to top](#top)

---

<a id="end-to-end-e2e"></a>

### End-to-End (E2E)

**Goal:** exercise the installed CLI as a user would.

Scope: plugin lifecycle (scaffold/install/list/info/check/uninstall), REPL sessions, DI graphs, HTTP API endpoints, performance-ish paths (history).

Technique: subprocess runs + shape fixtures (JSON/YAML), timing thresholds where meaningful.

[Back to top](#top)

---

<a id="fixtures--data"></a>

## Fixtures & Data

- Global fixtures manage tmp dirs, env overrides, deterministic clocks.  
- E2E **shape** fixtures (e.g., `di_shape.json`, `history_shape.yaml`) guard contract stability.  
- Redaction helpers ensure secrets never leak in snapshots.

[Back to top](#top)

---

<a id="conventions--markers"></a>

## Conventions & Markers

- **Naming:** `test_*.py` / `test_*` functions.  
- **Style:** pytest idioms; parametrize for matrix cases; minimal mocks.  
- **Markers:**  
  - `@pytest.mark.slow` — perf/network-heavy  
  - `@pytest.mark.e2e` — full CLI/subprocess  
  - `@pytest.mark.asyncio` — async flows

[Back to top](#top)

---

<a id="running-tests"></a>

## Running Tests

```bash
# Everything (via Makefile)
make test

# Plain pytest
pytest -q

# Tox (multi-Python; mirrors CI)
tox -q -p auto          # or: tox -e py311,py312,py313
```

Extras:

```bash
# With coverage report
pytest --cov=bijux_cli --cov-report=term-missing

# Stop early / short tracebacks
pytest -x --tb=short
```

[Back to top](#top)

---

<a id="selecting-scoping-tests"></a>

## Selecting/Scoping Tests

```bash
# Only unit
pytest tests/unit -q

# Only E2E
pytest -m e2e -q

# Exclude slow
pytest -m "not slow" -q

# Keyword selection
pytest -k "plugins and not uninstall" -q
```

[Back to top](#top)

---

<a id="coverage--quality-gates"></a>

## Coverage & Quality Gates

* **Code coverage:** **≥ 98% overall** (enforced in CI).
  Generate locally:

  ```bash
  pytest --cov=bijux_cli --cov-report=term-missing
  pytest --cov=bijux_cli --cov-report=html && open htmlcov/index.html
  ```
* **Docs/style:** `ruff` (lint+format), `pydocstyle`, `interrogate`.
* **Types:** `mypy`, `pyright` (strict).
* **Security/hygiene:** `bandit`, `pip-audit`, `reuse`, `codespell`, `deptry`, `radon`.

[Back to top](#top)

---

<a id="ci-pipeline"></a>

## CI Pipeline

**GitHub Actions** on pushes/PRs:

* **Matrix:** Python 3.11–3.13; Node 20 + Java 17 for OpenAPI/Redoc tooling.
* **Stages:** lint → type → unit/integration/functional → E2E → docs (strict) → security.
* **Gates:** any failure blocks merge; artifacts include coverage + logs.

[Back to top](#top)

---

<a id="mutation--property-based-testing"></a>

## Mutation & Property-Based Testing

* **Mutation:** **Cosmic Ray** / **mutmut** validate assertion strength (serializer, retry/backoff, config).

  ```bash
  make mutation
  ```
* **Property-based:** **Hypothesis** explores edge cases (serializer inputs, option matrices).

[Back to top](#top)

---

<a id="performance-flakes-determinism"></a>

## Performance, Flakes, Determinism

* Mark perf-sensitive tests `slow`; assert upper bounds when meaningful.
* Avoid real sleeps; mock clocks/backoffs or patch `asyncio.sleep`.
* Seed PRNG; no reliance on wall-clock randomness.
* Flakes are bugs—quarantine briefly with markers only until fixed.

[Back to top](#top)

---

<a id="contributing-new-tests"></a>

## Contributing New Tests

* **Pick the right layer** (unit vs integration vs E2E).
* **Prefer parametrization** over copy-pasting.
* **Assert shapes & side-effects** (stdout/stderr, files, telemetry).
* **Keep fixtures local** unless broadly reusable.
* Avoid network & real home dirs—use tmp paths + env overrides.
* Add **docs/examples** when behavior is user-visible.

[Back to top](#top)

---

<a id="quick-commands-cheat-sheet"></a>

## Quick Commands Cheat Sheet

```bash
# Fast local loop
pytest tests/unit -q

# Functional + E2E only
pytest -m "functional or e2e" -q

# Coverage HTML
pytest --cov=bijux_cli --cov-report=html && open htmlcov/index.html

# Tox across Pythons
tox -q -p auto

# Lint+type before PR
make lint quality
```

[Back to top](#top)

---

<a id="links"></a>

## Links

* **Tests directory:** [https://github.com/bijux/bijux-cli/tree/main/tests](https://github.com/bijux/bijux-cli/tree/main/tests)
* **CI runs:** [https://github.com/bijux/bijux-cli/actions](https://github.com/bijux/bijux-cli/actions)

[Back to top](#top)
