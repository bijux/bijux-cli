# Developer Setup
<a id="top"></a>

One place for all the extra tools this repo uses—what they do, where they’re configured, and how to run them locally or in CI.

---

## Table of Contents

* [At a Glance](#at-a-glance)
* [Prereqs & Versions](#prereqs--versions)
* [Formatting & Linting](#formatting--linting)
* [Typing](#typing)
* [Tests & Coverage](#tests--coverage)
* [Mutation & Property Tests](#mutation--property-tests)
* [Security & Supply Chain](#security--supply-chain)
* [Docs Toolchain](#docs-toolchain)
* [API Tooling](#api-tooling)
* [Packaging & Releases](#packaging--releases)
* [Git Hooks & Commit Hygiene](#git-hooks--commit-hygiene)
* [CI Orchestration](#ci-orchestration)
* [Config Files (Map)](#config-files-map)

[Back to top](#top)

---

<a id="at-a-glance"></a>

## At a Glance

* **Make** is the front door; **tox** mirrors CI envs.
* Tool configs live in `config/` and root dotfiles.
* Most tasks have a 1:1 `make` target (`make help` shows all).

```bash
make install   # bootstrap virtualenv, pin tools
tox -av        # list CI-mirrored environments
```

[Back to top](#top)

---

<a id="prereqs--versions"></a>

## Prereqs & Versions

* Python **3.11–3.13** (recommend `pyenv`)
* Node.js (API validation / docs assets)
* Java 17 (OpenAPI Generator CLI)
* GNU Make, git

[Back to top](#top)

---

<a id="formatting--linting"></a>

## Formatting & Linting

Run both locally and in CI:

```bash
make lint         # ruff format+lint, mypy, pyright, codespell, radon, pydocstyle (+ pytype ≤ 3.12)
make quality      # vulture, deptry, reuse, interrogate, codespell, radon
```

**How `make lint` works (per current `makefiles/lint.mk`):**

* Runs **per file** across `src/bijux_cli` and `tests`:

  * `ruff format` (code style)
  * `ruff check --fix` (lint autofix, config at `config/ruff.toml`)
  * `mypy --strict` (config at `config/mypy.ini`)
  * `codespell -I config/bijux.dic`
  * `pyright --project config/pyrightconfig.json`
  * `radon cc -s -a` (complexity)
  * `pydocstyle --convention=google` (docstring style)
* Then runs **Pytype** once per directory **only on Python ≤ 3.12**; it is **skipped on 3.13+**.

<details>
<summary>Make: Lint (<code>makefiles/lint.mk</code>)</summary>

```make
--8<-- "makefiles/lint.mk"
```

</details>

<details>
<summary>Make: Quality (<code>makefiles/quality.mk</code>)</summary>

```make
--8<-- "makefiles/quality.mk"
```

</details>

<details>
<summary>Ruff config (<code>config/ruff.toml</code>)</summary>

```
--8<-- "config/ruff.toml"
```

</details>

<details>
<summary>Deptry (<code>pyproject.toml</code>)</summary>

```toml
--8<-- "pyproject.toml:deptry"
```

</details>

<details>
<summary>Interrogate (<code>pyproject.toml</code>)</summary>

```toml
--8<-- "pyproject.toml:interrogate"
```

</details>

[Back to top](#top)

---

<a id="typing"></a>

## Typing

Strict static typing with **mypy** and **pyright**; **pytype** runs when supported (≤3.12), skipped on 3.13+.

```bash
make lint   # includes mypy + pyright + pytype
```

<details>
<summary>Mypy config (<code>config/mypy.ini</code>)</summary>

```ini
--8<-- "config/mypy.ini"
```

</details>

<details>
<summary>Pyright config (<code>config/pyrightconfig.json</code>)</summary>

```
--8<-- "config/pyrightconfig.json"
```

</details>

[Back to top](#top)

---

<a id="tests--coverage"></a>

## Tests & Coverage

* **pytest** + **pytest-cov**, overall coverage gate **≥ 98%** (see config).
* Per-layer markers for fast selection (unit/integration/functional/e2e).

```bash
make test
pytest --cov=bijux_cli --cov-report=term-missing
pytest --cov=bijux_cli --cov-report=html && open htmlcov/index.html
```

<details>
<summary>Coverage config (<code>config/coveragerc.ini</code>)</summary>

```ini
--8<-- "config/coveragerc.ini"
```

</details>

<details>
<summary>Pytest defaults (<code>pytest.ini</code>)</summary>

```ini
--8<-- "pytest.ini"
```

</details>

[Back to top](#top)

---

<a id="mutation--property-tests"></a>

## Mutation & Property Tests

* **mutmut** and **Cosmic Ray** validate assertion strength.
* **Hypothesis** for property-based tests.

```bash
make mutation
```

<details>
<summary>Cosmic Ray config (<code>config/cosmic-ray.toml</code>)</summary>

```
--8<-- "config/cosmic-ray.toml"
```

</details>

[Back to top](#top)

---

<a id="security--supply-chain"></a>

## Security & Supply Chain

* **bandit** (SAST), **pip-audit** (CVE scan)
* **CycloneDX SBOM** generation (`artifacts/sbom.json`)
* SPDX compliance via **REUSE**

```bash
make security   # bandit + pip-audit
make sbom       # writes artifacts/sbom.json
```

<details>
<summary>Make: Security (<code>makefiles/security.mk</code>)</summary>

```make
--8<-- "makefiles/security.mk"
```

</details>

<details>
<summary>Make: SBOM (<code>makefiles/sbom.mk</code>)</summary>

```make
--8<-- "makefiles/sbom.mk"
```

</details>

<details>
<summary>REUSE config (<code>REUSE.toml</code>)</summary>

```
--8<-- "REUSE.toml"
```

</details>

[Back to top](#top)

---

<a id="docs-toolchain"></a>

## Docs Toolchain

* **MkDocs (Material)** + **mkdocstrings** + **literate-nav**.
* `scripts/helper_mkdocs.py` copies top-level Markdown into `docs/`, generates `reference/**` API pages, `nav.md`, and ensures `{#top}` anchors.

```bash
make docs
make docs-serve
```

<details>
<summary>MkDocs config (<code>mkdocs.yml</code>)</summary>

```yaml
--8<-- "mkdocs.yml"
```

</details>

<details>
<summary>Docs generator (<code>scripts/helper_mkdocs.py</code>)</summary>

```python
--8<-- "scripts/helper_mkdocs.py"
```

</details>

[Back to top](#top)

---

<a id="api-tooling"></a>

## API Tooling

* Validation: **Prance**, **openapi-spec-validator**, **Redocly**
* Codegen compatibility: **OpenAPI Generator CLI**
* **Schemathesis** contract tests against a running server

```bash
.venv/bin/uvicorn bijux_cli.httpapi:app --host 0.0.0.0 --port 8000 &
make api
```

<details>
<summary>Make: API (<code>makefiles/api.mk</code>)</summary>

```make
--8<-- "makefiles/api.mk"
```

</details>

<details>
<summary>OpenAPI schema (<code>api/v1/schema.yaml</code>)</summary>

```yaml
--8<-- "api/v1/schema.yaml"
```

</details>

[Back to top](#top)

---

<a id="packaging--releases"></a>

## Packaging & Releases

* Build backend: **hatch**
* Versioning: **hatch-vcs** (from git tags)
* PyPI long description: **hatch-fancy-pypi-readme**
* Changelog: **Towncrier** (fragments in `changelog.d/`)
* Conventional Commits: **Commitizen**
* Publishing: GitHub Actions `publish.yml` → `make publish`

```bash
make build
# tag with commitizen as vX.Y.Z, then push tags to trigger publish workflow
```

<details>
<summary>Hatch/metadata (<code>pyproject.toml</code>)</summary>

```toml
--8<-- "pyproject.toml:hatch"
```

</details>

[Back to top](#top)

---

<a id="git-hooks--commit-hygiene"></a>

## Git Hooks & Commit Hygiene

* **pre-commit** hooks (see `.pre-commit-config.yaml`)
* Conventional Commits enforced (Commitizen)
* Auto-fragment creator: `scripts/git-hooks/prepare-commit-msg`
* Guard requiring a fragment: `scripts/check-towncrier-fragment.sh`

```bash
pre-commit install
ln -sf ../../scripts/git-hooks/prepare-commit-msg .git/hooks/prepare-commit-msg
```

<details>
<summary>pre-commit config (<code>.pre-commit-config.yaml</code>)</summary>

```yaml
--8<-- ".pre-commit-config.yaml"
```

</details>

<details>
<summary>prepare-commit-msg hook (<code>scripts/git-hooks/prepare-commit-msg</code>)</summary>

```bash
--8<-- "scripts/git-hooks/prepare-commit-msg"
```

</details>

<details>
<summary>Towncrier fragment guard (<code>scripts/check-towncrier-fragment.sh</code>)</summary>

```bash
--8<-- "scripts/check-towncrier-fragment.sh"
```

</details>

[Back to top](#top)

---

<a id="ci-orchestration"></a>

## CI Orchestration

* **GitHub Actions**: main CI, docs deploy, and publish pipelines.
* **tox** mirrors Make targets for matrix runs.
* Makefile is modularized under `makefiles/*.mk`.

```bash
tox -q -p auto
```

<details>
<summary>Makefile (entrypoint) (<code>Makefile</code>)</summary>

```make
--8<-- "Makefile"
```

</details>

<details>
<summary>tox config (<code>tox.ini</code>)</summary>

```ini
--8<-- "tox.ini"
```

</details>

<details>
<summary>CI workflow: main (<code>.github/workflows/ci.yml</code>)</summary>

```yaml
--8<-- ".github/workflows/ci.yml"
```

</details>

<details>
<summary>CI workflow: docs (<code>.github/workflows/docs.yml</code>)</summary>

```yaml
--8<-- ".github/workflows/docs.yml"
```

</details>

<details>
<summary>CI workflow: publish (<code>.github/workflows/publish.yml</code>)</summary>

```yaml
--8<-- ".github/workflows/publish.yml"
```

</details>

[Back to top](#top)

---

<a id="config-files-map"></a>

## Config Files (Map)

* Lint: `config/ruff.toml`
* Types: `config/mypy.ini`, `config/pyrightconfig.json`
* Coverage: `config/coveragerc.ini`
* Mutation: `config/cosmic-ray.toml`
* Dictionary (codespell): `config/bijux.dic`
* Config notes: `config/README.md`
* CI: `.github/workflows/`, `tox.ini`, `pytest.ini`
* Docs: `mkdocs.yml`, `scripts/helper_mkdocs.py`
* Security & licensing: `REUSE.toml`
* Packaging & release: `pyproject.toml`

[Back to top](#top)
