# ADR 0004: Linting, Quality, and Security Toolchain

* **Date:** 2025-08-01
* **Status:** Accepted
* **Author:** Bijan Mousavi

---

## Context

We need a single, reproducible pipeline for code style, formatting, type-safety, complexity, documentation coverage, dead code, dependency hygiene, license compliance, and security checks — identical locally and in CI. Developers should be able to run:

```
make lint
make quality
make security
```

and get the same results everywhere.

We standardized on the following tools:

* **Ruff** for **formatting**, **import sorting**, and **linting** (with auto-fix where safe).
* **Mypy** and **Pytype** for static typing (Pytype runs where supported).
* **Pyright** for fast type checks (editor/CI parity).
* **Pydocstyle** (Google convention) for docstring style.
* **Interrogate** for documentation coverage.
* **Radon** for cyclomatic complexity.
* **Vulture** for dead code detection.
* **Deptry** for unused/incorrect dependencies.
* **REUSE** for SPDX license header compliance.
* **Bandit** for security static analysis.
* **pip-audit** for dependency vulnerability audits.

All configuration lives under `config/` (with a few root files like `REUSE.toml`), ensuring CI/local parity.

---

## Decision

### Makefile Targets

We enforce Makefile targets to run the full toolchain consistently.

<details>
<summary>Lint (<code>Makefile</code>)</summary>

```make
{% include-markdown "../../makefiles/lint.mk"  comments=false dedent=true %}
```

</details>

<details>
<summary>Quality (<code>Makefile</code>)</summary>

```make
{% include-markdown "../../makefiles/quality.mk"  comments=false dedent=true %}
```

</details>

<details>
<summary>Security (<code>Makefile</code>)</summary>

```make
{% include-markdown "../../makefiles/security.mk"  comments=false dedent=true %}
```

</details>


This setup supports whole-project runs as well as per-directory/per-file runs, with reasonable exclusions for generated or template content.

### Tool Configurations

The toolchain is driven by unified configs:

<details>
<summary>Ruff (<code>config/ruff.toml</code>)</summary>

```toml
{% include-markdown "../../config/ruff.toml" %}
```

</details>

<details>
<summary>Mypy (<code>config/mypy.ini</code>)</summary>

```ini
{% include-markdown "../../config/mypy.ini" %}
```

</details>

<details>
<summary>Pyright (<code>config/pyrightconfig.json</code>)</summary>

```json
{% include-markdown "../../config/pyrightconfig.json" %}
```

</details>

<details>
<summary>Deptry (<code>pyproject.toml</code>)</summary>

```toml
{% include-markdown "../../pyproject.toml" start="# deptry start" end="# deptry end" comments=false dedent=true %}
```

</details>

<details>
<summary>Interrogate (<code>pyproject.toml</code>)</summary>

```toml
{% include-markdown "../../pyproject.toml" start="# interrogate start" end="# interrogate end" comments=false dedent=true %}
```

</details>

<details>
<summary>REUSE (<code>REUSE.toml</code>)</summary>

```toml
{% include-markdown "../../REUSE.toml" %}
```

</details>

**Docstring Style Enforcement**
We mandate Google-style docstrings via Pydocstyle (enforced in Makefile):

```bash
pydocstyle --convention=google path/to/file.py
```

Interrogate enforces documentation coverage thresholds as configured.

---

## CI Integration

* `make lint` runs over `src/` and `tests/`.
* `make quality` and `make security` run project-wide.
* Any failure blocks the build; no overrides.

---

## Consequences

### Pros

* Uniform enforcement across the repo; no drift.
* **One tool (Ruff)** handles formatting, import sorting, and linting with fast auto-fixes.
* Strong typing via **Mypy**, **Pytype** (where supported), and **Pyright**.
* Doc style & coverage enforced via **Pydocstyle** + **Interrogate**.
* Maintainability boosted by **Vulture** (dead code), **Deptry** (deps), **Radon** (complexity).
* SPDX compliance via **REUSE**.
* Security posture improved through **Bandit** + **pip-audit**.
* All configs centralized under `config/`, ensuring local/CI parity.

### Cons

* Initial setup and periodic rule maintenance.
* Contributors must align with strict rules and workflow.

---

## Enforcement

* Code is accepted only if it passes all configured targets and checks in this ADR.
* Reviewers & CI must reject non-compliant changes (lint, quality, security, or config deviations).
* This policy is binding to preserve the integrity of the toolchain.
