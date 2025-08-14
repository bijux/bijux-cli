# Project Tree & Guide
<a id="top"></a>

A guided map of the Bijux CLI repository: what lives where, and why.

---

## Table of Contents

* [At a Glance](#at-a-glance)
* [Top-Level Files](#top-level-files)
* [Dotfiles & CI/CD](#dotfiles-cicd)
* [Source Code](#source-code)
  * [Commands](#commands)
  * [Services](#services)
  * [Contracts](#contracts)
  * [Core](#core)
  * [Infra](#infra)
  * [Root Modules](#root-modules)
* [API Schema](#api-schema)
* [Documentation Site](#documentation-site)
* [Configuration](#configuration)
* [Build & Automation](#build-automation)
* [Plugin Template](#plugin-template)
* [Tests](#tests)
* [Packaging & Tooling](#packaging-tooling)
* [Licensing & Governance](#licensing-governance)
* [Changelog & Releases](#changelog-releases)

[Back to top](#top)

---

<a id="at-a-glance"></a>
## At a Glance

```

.
├── api/                # OpenAPI schemas
├── config/             # Lint/type/security configs
├── docs/               # MkDocs site (Material theme)
├── makefiles/          # Makefile modules for tasks
├── plugin_template/    # Cookiecutter-compatible plugin scaffold
├── scripts/            # Helper scripts (hooks, docs generation)
├── src/bijux_cli/      # Library + CLI implementation
├── tests/              # Unit, integration, functional, E2E
└── .github/workflows/  # CI/CD pipelines (GitHub Actions)

```

[Back to top](#top)

---

<a id="top-level-files"></a>
## Top-Level Files

* `README.md` — overview and quickstart  
* `USAGE.md` — user guide  
* `TESTS.md` — testing philosophy & how to run tests  
* `PROJECT_TREE.md` — this guided map (mirrors `docs/project_tree.md` for the site)  
* `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md` — community & policy  
* `CHANGELOG.md`, `changelog.d/` — release notes + fragments  
* `pyproject.toml`, `tox.ini`, `pytest.ini` — packaging & test config  
* `mkdocs.yml` — docs site config  
* `LICENSES/` — MIT and CC0 texts  
* `CITATION.cff` — citation metadata  
* `REUSE.toml` — REUSE licensing compliance  
* `Makefile` — task entrypoint (delegates to `makefiles/`)

[Back to top](#top)

---

<a id="dotfiles-cicd"></a>
## Dotfiles & CI/CD

Hidden files/directories that govern editor behavior, linting, hooks, and pipelines.

```

.
├── .editorconfig
├── .gitattributes
├── .gitignore
├── .gitlab-ci.yml
├── .pre-commit-config.yaml
└── .github/
    └── workflows/
        ├── ci.yml     # tests, lint, type-check, security, build
        └── docs.yml   # docs build/deploy

```

**Pre-commit (recommended):**
```bash
pipx install pre-commit || python -m pip install -U pre-commit
pre-commit install
pre-commit run --all-files
```

[Back to top](#top)

---

<a id="source-code"></a>

## Source Code

Path: `src/bijux_cli/` — CLI entrypoints, DI kernel, services, contracts, and infrastructure.

```
src/bijux_cli/
├── __init__.py
├── __main__.py             # python -m bijux_cli
├── __version__.py
├── api.py                  # HTTP API wiring (if enabled)
├── cli.py                  # Typer root app (entrypoint)
├── httpapi.py              # HTTP server endpoints
├── commands/               # User-facing commands
├── services/               # Business logic implementations
├── contracts/              # Typed interfaces (protocols)
├── core/                   # Engine, DI, enums, exceptions, paths
└── infra/                  # Emitters, serializers, telemetry, retry, process
```

*`py.typed` files (PEP 561) are included across packages to advertise type completeness.*

[Back to top](#top)

<a id="commands"></a>

### Commands

End-user commands and subcommands. Each file is a Typer command module.

```
src/bijux_cli/commands/
├── audit.py  docs.py  doctor.py  help.py  repl.py  sleep.py
├── status.py  utilities.py  version.py
├── config/   # clear/export/get/list_cmd/load/reload/service/set/unset
├── dev/      # di/list_plugins/service
├── history/  # clear/service
├── memory/   # clear/delete/get/list/service/set/utils
└── plugins/  # check/info/install/list/scaffold/uninstall/utils
```

* **Top-level commands**: operational functions (`doctor`, `status`, `audit`, …).
* **Namespace commands** group related operations and include a `service.py` adapter.

[Back to top](#top)

<a id="services"></a>

### Services

Concrete implementations behind commands. Orchestrate work, depend on `contracts` and `infra`.

```
src/bijux_cli/services/
├── audit.py  config.py  docs.py  doctor.py
├── history.py  memory.py  utils.py
└── plugins/ (entrypoints.py, groups.py, hooks.py, registry.py)
```

[Back to top](#top)

<a id="contracts"></a>

### Contracts

Typed interfaces (protocols/ABCs) consumed by services — clean DI seams.

```
src/bijux_cli/contracts/
audit.py  config.py  context.py  docs.py  doctor.py  emitter.py
history.py  memory.py  observability.py  process.py  registry.py
retry.py  serializer.py  telemetry.py
```

[Back to top](#top)

<a id="core"></a>

### Core

Framework plumbing: engine loop, DI kernel, exceptions, enums, and path helpers.

```
src/bijux_cli/core/
constants.py  context.py  di.py  engine.py  enums.py  exceptions.py  paths.py
```

[Back to top](#top)

<a id="infra"></a>

### Infra

Foundational adapters/utilities used across services.

```
src/bijux_cli/infra/
emitter.py  observability.py  process.py  retry.py  serializer.py  telemetry.py
```

[Back to top](#top)

<a id="root-modules"></a>

### Root Modules

* `cli.py` — Typer app creation and command registration
* `api.py` / `httpapi.py` — HTTP API composition (if used)
* `__main__.py` — module entry (`python -m bijux_cli`)
* `__version__.py` — central version string

[Back to top](#top)

---

<a id="api-schema"></a>

## API Schema

```
api/
└── v1/
    └── schema.yaml   # OpenAPI spec for the HTTP API
```

Source of truth for the public HTTP API (if enabled). Used for validation and documentation.

[Back to top](#top)

---

<a id="documentation-site"></a>

## Documentation Site

MkDocs (Material) site; build with `make docs-serve` / `mkdocs build`.

```
docs/
├── index.md            # Home (wraps README)
├── usage.md            # User Guide (wraps USAGE)
├── tests.md            # Testing overview (wraps TESTS.md)
├── project_tree.md     # This guide (wraps PROJECT_TREE.md)
├── changelog.md        # Wraps CHANGELOG.md
├── security.md         # Wraps SECURITY.md
├── contributing.md     # Wraps CONTRIBUTING.md
├── code_of_conduct.md  # Wraps CODE_OF_CONDUCT.md
├── license.md          # Wraps LICENSES/MIT.txt
├── community.md        # Community landing (overview page)
├── ADR/                # Architecture Decision Records
├── assets/             # Logos, CSS (Material overrides in assets/styles/extra.css)
├── overrides/          # Jinja2 overrides for Material theme
└── nav.md              # Generated by helper script
```

> `scripts/helper_mkdocs.py` generates API reference pages and the full navigation at build time.

[Back to top](#top)

---

<a id="configuration"></a>

## Configuration

Centralized tool configs:

```
config/
├── bijux.dic             # custom dictionary
├── mypy.ini              # type checking
├── pyrightconfig.json    # pyright settings
├── ruff.toml             # lint rules (ruff)
├── coveragerc.ini        # coverage config
├── cosmic-ray.toml       # mutation testing
└── README.md             # notes about configs
```

[Back to top](#top)

---

<a id="build-automation"></a>

## Build & Automation

**Makefile modules**

```
makefiles/
api.mk  build.mk  changelog.mk  citation.mk  dictionary.mk
docs.mk hooks.mk  lint.mk  mutation.mk  publish.mk  quality.mk
sbom.mk security.mk  test.mk
```

**Helper scripts**

```
scripts/
├── helper_mkdocs.py             # generates docs nav + API reference
├── helper_comments.py           # doc/comment utilities
├── check-towncrier-fragment.sh  # changelog fragments guard
├── git-hooks/prepare-commit-msg # conventional commit assist
└── README.md
```

[Back to top](#top)

---

<a id="plugin-template"></a>

## Plugin Template

Cookiecutter-ready skeleton for third-party plugins.

```
plugin_template/
├── README.md  __init__.py  cookiecutter.json  pyproject.toml
└── {{cookiecutter.project_slug}}/
    ├── __init__.py
    ├── plugin.json
    └── plugin.py
```

* `plugin.json` — plugin metadata & entry points
* `plugin.py` — plugin’s main module

[Back to top](#top)

---

<a id="tests"></a>

## Tests

Four layers: **unit**, **integration**, **functional**, **E2E**.
See also **docs/tests.md** and the repo file: [https://github.com/bijux/bijux-cli/blob/main/TESTS.md](https://github.com/bijux/bijux-cli/blob/main/TESTS.md).

```
tests/
├── unit/          # Fast, isolated component tests
├── integration/   # Subsystem wiring, DI, flows
├── functional/    # User-facing behavior/flags/output
└── e2e/           # Full CLI runs + fixtures
```

Highlights:

* `e2e/test_fixtures/` — JSON/YAML expected outputs (shape tests)
* Namespaced E2E (e.g., `plugins/`, `repl/`, `history/`) mirror command areas
* Root tests (`unit/root/`) cover entry points: `__main__`, `cli`, `api`, `httpapi`

Run examples:

```bash
make test           # all tests
make test-unit      # unit only
make test-e2e       # end-to-end only
```

[Back to top](#top)

---

<a id="packaging-tooling"></a>

## Packaging & Tooling

* `pyproject.toml` — build system, dependencies, entry points
* `tox.ini` — matrix/env automation
* `pytest.ini` — Pytest defaults
* `package.json`, `package-lock.json` — optional JS tooling for docs/assets
* `REUSE.toml` — REUSE licensing compliance

[Back to top](#top)

---

<a id="licensing-governance"></a>

## Licensing & Governance

* `LICENSES/` — MIT and CC0 texts
* `CODE_OF_CONDUCT.md` — community guidelines
* `CONTRIBUTING.md` — how to contribute
* `SECURITY.md` — vulnerability reporting policy
* `CITATION.cff` — citation metadata

[Back to top](#top)

---

<a id="changelog-releases"></a>

## Changelog & Releases

* `CHANGELOG.md` — curated release notes
* `changelog.d/` — fragment files (Towncrier-style)
* CI enforces fragment presence for PRs (see `scripts/check-towncrier-fragment.sh`)

[Back to top](#top)

---

**Tip:** New to the codebase? Start at `src/bijux_cli/cli.py`, jump to the command under `src/bijux_cli/commands/`, then follow into its corresponding service and contract.
