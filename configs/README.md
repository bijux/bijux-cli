# Configuration Layout

This directory contains repository-wide configuration used by Rust and Python tooling.
Python-specific tooling configuration is centralized under `configs/python`.

---

## Python

| File                              | Tool / Purpose                                                     |
|-----------------------------------|--------------------------------------------------------------------|
| **`python/pyproject.toml`**       | Python workspace package metadata and tooling settings             |
| **`python/tox.ini`**              | Tox environments used by CI and local validation                   |
| **`python/pytest.ini`**           | Pytest collection and coverage behavior                            |
| **`python/coveragerc.ini`**       | Coverage.py configuration                                          |
| **`python/mypy.ini`**             | Mypy strict static type checking                                   |
| **`python/ruff.toml`**            | Ruff linting, formatting, and import sorting rules                |

## Shared

| File                              | Tool / Purpose                                                     |
|-----------------------------------|--------------------------------------------------------------------|
| **`allowlists/*.toml`**           | Centralized policy allowlists consumed by maintainer automation    |
| **`rust/*.toml`**                 | Rust formatting, linting, dependency audit, and test profiles      |
| **`status/*.json`**               | Baseline status inputs for maintainer checks                       |
