# Contributing to `bijux-cli`

This doc is the single source of truth for local setup, workflows, API validation, and PR rules. If you follow this, your changes should pass CI on the first try. 🛠️

---

## 1) Quick Start

**Prereqs**

* Python **3.11 / 3.12 / 3.13** (`pyenv` recommended)
* **GNU Make**
* **Node.js + npm** (for API validation tooling)
* Optional: **pre-commit** (to catch issues before pushing)

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
* ✘ Fail → jump to [Troubleshooting](#11-troubleshooting)

---

## 2) Daily Workflow

* Everything runs inside **.venv/**
* No global installs after `make install`
* Make targets mirror CI jobs 1:1

**Core targets**

| Target          | What it does                                                                |
|-----------------|-----------------------------------------------------------------------------|
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
make docs-serve          # local docs server
# make docs-deploy       # if you have perms
```

---

## 3) API Development

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

---

## 4) Docs

* Config: `mkdocs.yml` (Material, **strict**)
* Build: `make docs`
* Serve: `make docs-serve`
* Deploy: `make docs-deploy` (if authorized)

---

## 5) Tests & Coverage

* Run all tests: `make test`
* Focused run: `pytest -k "<expr>" -q`
* Coverage report: HTML in `htmlcov/`
  (Project enforces a high bar; keep it green.)

---

## 6) Style, Types, Hygiene

* **Formatting:** `ruff format` (enforced in `make lint`)
* **Linting:** `ruff`
* **Types:** `mypy` (strict) + `pyright` (strict)
* **Complexity:** `radon`
* **Docstrings:** `interrogate` (keep modules ≥ target thresholds)

Run them all:

```bash
make lint
```

---

## 7) Security & Supply Chain

```bash
make security   # bandit + pip-audit
make sbom       # CycloneDX, saved to artifacts/
```

* No secrets in code or tests
* Keep dependency pins sane; document any suppressions

---

## 8) Tox Envs (mirror CI)

| Env                         | Runs            |
|-----------------------------|-----------------|
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

---

## 9) Commits & PRs

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

---

## 10) Pre-Commit

```bash
pre-commit install
```

Runs critical checks locally (format, lint, commit message validation, etc.).

---

## 11) Troubleshooting

* **Missing Node.js** → required for API validation tools
* **Docs fail** → MkDocs is strict; fix broken links/includes
* **pytype on Python > 3.12** → skipped automatically
* **Port in use for API tests** → kill old `uvicorn` or use a different port

---

## 12) Community & Conduct

Be kind and constructive. See the **Code of Conduct** in the docs site. If you see something off, let us know.

---

**Build well. Break nothing.** 