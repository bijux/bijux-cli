# Project Configuration

This directory contains all configuration files for **`bijux-cli`** development and quality assurance.
It is the **single source of truth** for code formatting, linting, type checking, testing, and security rules — ensuring every contributor works in a **predictable, consistent** environment.

Centralizing these configurations ensures:

* **One place** to update standards
* **Identical behavior** across local and CI builds

---

## Quick Reference

| File                              | Tool / Purpose                                                     |
|-----------------------------------|--------------------------------------------------------------------|
| **`bijux.dic`**                   | PyCharm / Codespell — custom dictionary for project-specific terms |
| **`cosmic-ray.toml`**             | Cosmic Ray — mutation testing configuration                        |
| **`coveragerc.ini`**              | Coverage.py — coverage measurement rules                           |
| **`mypy.ini`**                    | Mypy — strict static type checking                                 |
| **`pyrightconfig.json`**          | Pyright — fast static type analysis                                |
| **`ruff.toml`**                   | Ruff — linting, formatting, and isort rules                        |

