# Changelog

All notable changes to **Bijux CLI** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## [Unreleased]

### Added

* (add new entries via Towncrier fragments in `changelog.d/`)

### Changed

* (add here)

### Fixed

* (add here)

---

<!-- towncrier start -->

## [0.1.0] – 2025-08-12

### Added

* **Core runtime**

    * Implemented Dependency Injection kernel, REPL shell, plugin loader, telemetry hooks, and shell completion (bash/zsh/fish).
    * Added core modules: `api`, `cli`, `httpapi`, `core/{constants,context,di,engine,enums,exceptions,paths}`.

* **Contracts layer** (`contracts/`)

    * Defined protocols for `audit`, `config`, `context`, `docs`, `doctor`, `emitter`, `history`,
      `memory`, `observability`, `process`, `registry`, `retry`, `serializer`, `telemetry`.
    * Added `py.typed` markers for downstream type checking.

* **Services layer**

    * Implemented concrete services for `audit`, `config`, `docs`, `doctor`, `history`, `memory`.
    * Built plugin subsystem: `plugins/{entrypoints,groups,hooks,registry}`.

* **Infra layer** (`infra/`)

    * Implemented `emitter`, `observability`, `process`, `retry`, `serializer`, `telemetry`.

* **Command suite**

    * Added top-level commands: `audit`, `docs`, `doctor`, `help`, `repl`, `sleep`, `status`, `version`.
    * Added `config/` commands: `clear`, `export`, `get`, `list`, `load`, `reload`, `set`, `unset`, `service`.
    * Added `dev/` commands: `di`, `list-plugins`, `service`.
    * Added `history/` commands: `clear`, `service`.
    * Added `memory/` commands: `clear`, `delete`, `get`, `list`, `set`, `service`.
    * Added `plugins/` commands: `check`, `info`, `install`, `list`, `scaffold`, `uninstall`.

* **Structured output & flags**

    * Added JSON/YAML output via `--format`, pretty printing, and deterministic global flag precedence ([ADR-0002](https://bijux.github.io/bijux-cli/ADR/0002-global-flags-precedence/)).

* **API contract validation & testing**

    * Automated lint/validation of `api/*.yaml` with Prance, OpenAPI Spec Validator, Redocly, and OpenAPI Generator.
    * Added **Schemathesis** contract testing against the running server.
    * Pinned OpenAPI Generator CLI version via `OPENAPI_GENERATOR_VERSION` and automated Node.js toolchain setup in Makefile.

* **Documentation tooling**

    * Integrated MkDocs (Material), mkdocstrings, literate-nav, and ADR index generation.

* **Quality & security pipeline**

    * Added formatting/linting: `ruff` (+format).
    * Added typing: `mypy`, `pyright`, `pytype`.
    * Added docs style/coverage: `pydocstyle`, `interrogate`.
    * Added code health: `vulture`, `deptry`, `radon`, `codespell`, `reuse`.
    * Added security: `bandit`, `pip-audit`.
    * Added mutation testing: `mutmut`, `cosmic-ray`.

* **SBOM**

    * Generated CycloneDX JSON for prod/dev dependencies via `make sbom` (uses `pip-audit`).

* **Citation**

    * Validated `CITATION.cff` and added export to BibTeX/RIS/EndNote formats via `make citation`.

* **Makefile architecture**

    * Modularized the Makefile into `makefiles/*.mk` for maintainability and clear separation of concerns.
    * Centralized all developer workflows (`test`, `lint`, `quality`, `security`, `api`, `docs`, `build`, `sbom`, `citation`, `changelog`, `publish`) in one consistent interface.
    * Added `bootstrap` target for idempotent virtualenv setup and Git hook installation from `scripts/git-hooks` (skips re-installation if already linked).
    * Added `all-parallel` target to run independent checks (`quality`, `security`, `api`, `docs`) concurrently for faster CI/CD.
    * Added `make help` for self-documenting targets with grouped sections.
    * Provided helper macros (`run_tool`, `read_pyproject_version`) to standardize tooling invocation.

* **pre-commit**

    * Added hygiene hooks: `ruff-format` + `ruff`, `mdformat`, and `codespell`.
      * Enforced Conventional Commits via **commitizen** `commit-msg` hook.

* **tox orchestration**

    * Configured multi-Python test envs (`py311`, `py312`, `py313`).
    * Mapped Makefile workflows into tox envs (`lint`, `quality`, `security`, `api`, `docs`, `build`, `sbom`, `changelog`, `citation`) to ensure reproducibility.
    * Passed `MAKEFLAGS` to execute Makefile targets inside tox-managed virtualenvs.

* **Continuous Integration**

    * Added **GitHub Actions** workflow running tox across Python versions with Node.js 20 and Java 17 for API checks.
    * Added **GitLab CI** sample mirroring the GitHub workflow (tox-driven) with artifacts for coverage and API logs.
    * CI/CD pipelines directly leverage the modularized Makefile for consistent local/CI behavior.

* **Packaging / PyPI page**

    * Built dynamic long description via **hatch-fancy-pypi-readme** from **README.md** and **CHANGELOG.md** for PyPI/TestPyPI.
    * Packaged with `LICENSES/`, `REUSE.toml`, `CITATION.cff`, and `py.typed` included in source distributions.

### Changed

* Released initial public version.

### Fixed

* None

[Unreleased]: https://github.com/bijux/bijux-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bijux/bijux-cli/releases/tag/v0.1.0
