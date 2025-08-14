# Contributing to Bijux CLI
<a id="top"></a>

This guide is the single source of truth for local setup, workflows, API validation, and PR rules. Follow it to ensure your changes pass CI seamlessly.️

---

## Table of Contents

- [Quick Start](#quick-start)
- [Daily Workflow](#daily-workflow)
- [API Development](#api-development)
- [Docs](#docs)
- [Tests & Coverage](#tests-coverage)
- [Style, Types, Hygiene](#style-types-hygiene)
- [Security & Supply Chain](#security-supply-chain)
- [Tox Envs (Mirror CI)](#tox-envs-mirror-ci)
- [Commits & PRs](#commits-prs)
- [Pre-Commit](#pre-commit)
- [Troubleshooting](#troubleshooting)
- [Community & Conduct](#community-conduct)

[Back to top](#top)

---

<a id="quick-start"></a>

## Quick Start

**Prereqs**

- Python **3.11 / 3.12 / 3.13** (`pyenv` recommended)
- **GNU Make**
- **Node.js + npm** (for API validation tooling)
- Optional: **pre-commit** (to catch issues before pushing)

**Setup**

```bash
git clone https://github.com/bijux/bijux-cli.git
cd bijux-cli
make PYTHON=python3.11 install
source .venv/bin/activate
# optional but recommended
pre-commit install
```

**Sanity check**

```bash
make lint test docs api
```

* ✔ Pass → your env matches CI
* ✘ Fail → jump to [Troubleshooting](#troubleshooting)

[Back to top](#top)

---

<a id="daily-workflow"></a>

## Daily Workflow

* Everything runs inside **.venv/**
* No global installs after `make install`
* Make targets mirror CI jobs 1:1

**Core targets**

| Target          | What it does                                                                |
| --------------- | --------------------------------------------------------------------------- |
| `make test`     | `pytest` + coverage (HTML in `htmlcov/`)                                    |
| `make lint`     | Format (ruff), lint (ruff), type-check (mypy/pyright), complexity (radon)   |
| `make quality`  | Dead code (vulture), deps hygiene (deptry), REUSE, docstrings (interrogate) |
| `make security` | Bandit + pip-audit                                                          |
| `make api`      | OpenAPI lint + generator compat + Schemathesis contract tests               |
| `make docs`     | Build MkDocs (strict)                                                       |
| `make build`    | Build sdist + wheel                                                         |
| `make sbom`     | CycloneDX SBOM → `artifacts/sbom.json`                                      |
| `make mutation` | Mutation testing (Cosmic Ray + Mutmut)                                      |

**Handy helpers**

```bash
make lint-file file=path/to/file.py
make docs-serve    # local docs server
# make docs-deploy # if you have perms
```

[Back to top](#top)

---

<a id="api-development"></a>

## API Development

**Schema:** `api/v1/schema.yaml`
**Tooling:** Prance, OpenAPI Spec Validator, Redocly, OpenAPI Generator, Schemathesis

**Validate locally**

```bash
.venv/bin/uvicorn bijux_cli.httpapi:app --host 0.0.0.0 --port 8000 &
make api
```

**Contract rules**

* Errors use **RFC 7807 Problem JSON**
* Response shapes and pagination are stable or versioned
* Breaking changes require a versioned path **and** a changelog entry

[Back to top](#top)

---

<a id="docs"></a>

## Docs

* Config: `mkdocs.yml` (Material, **strict**)
* Build: `make docs`
* Serve: `make docs-serve`
* Deploy: `make docs-deploy` (if authorized)

[Back to top](#top)

---

<a id="tests-coverage"></a>

## Tests & Coverage

* Run all tests: `make test`
* Focused run: `pytest -k "<expr>" -q`
* Coverage report: HTML in `htmlcov/`
* **Project bar:** \~**2,600+ tests** with **≥98%** coverage across unit/integration/functional/E2E. Keep it green.

[Back to top](#top)

---

<a id="style-types-hygiene"></a>

## Style, Types, Hygiene

* **Formatting:** `ruff format` (enforced in `make lint`)
* **Linting:** `ruff`
* **Types:** `mypy` (strict) + `pyright` (strict)
* **Complexity:** `radon`
* **Docstrings:** `interrogate` (meet configured thresholds)

Run them all:

```bash
make lint
```

[Back to top](#top)

---

<a id="security-supply-chain"></a>

## Security & Supply Chain

```bash
make security  # bandit + pip-audit
make sbom      # CycloneDX, saved to artifacts/
```

* No secrets in code or tests
* Keep dependency pins sane; document any suppressions

[Back to top](#top)

---

<a id="tox-envs-mirror-ci"></a>

## Tox Envs (Mirror CI)

| Env                         | Runs            |
| --------------------------- | --------------- |
| `py311` / `py312` / `py313` | `make test`     |
| `lint`                      | `make lint`     |
| `quality`                   | `make quality`  |
| `security`                  | `make security` |
| `api`                       | `make api`      |
| `docs`                      | `make docs`     |
| `build`                     | `make build`    |
| `sbom`                      | `make sbom`     |

List all:

```bash
tox -av
```

[Back to top](#top)

---

<a id="commits-prs"></a>

## Commits & PRs

### Conventional Commits (required)

```
<type>(<scope>): <description>
```

**Types:** `feat` `fix` `docs` `style` `refactor` `test` `chore`

**Example**

```
feat(plugins): add plugin scaffolding command
```

**Breaking changes** must include:

```
BREAKING CHANGE: <explanation>
```

> Commit messages are validated (Commitizen via pre-commit hook).

### PR Checklist

1. Branch from `main`
2. Run:

   ```bash
   make lint test api docs
   ```
3. Ensure Conventional Commits
4. Open PR with clear summary & rationale

[Back to top](#top)

---

<a id="pre-commit"></a>

## Pre-Commit

```bash
pre-commit install
```

Runs critical checks locally (format, lint, commit message validation, etc.).

[Back to top](#top)

---

<a id="troubleshooting"></a>

## Troubleshooting

* **Missing Node.js** → required for API validation tools
* **Docs fail** → MkDocs is strict; fix broken links/includes
* **pytype on Python > 3.12** → skipped automatically
* **Port in use for API tests** → kill old `uvicorn` or use a different port

[Back to top](#top)

---

<a id="community-conduct"></a>

## Community & Conduct

Be kind and constructive. See the **Code of Conduct** in the docs site. If you see something off, let us know.

[Back to top](#top)

---

**Build well. Break nothing.**
