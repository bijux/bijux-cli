# Bijux CLI
<a id="top"></a>

**A modern, predictable CLI framework for Python** — strict global flag precedence, first-class **plugins**, a **DI kernel**, and an interactive **REPL**. Build robust, extensible command-line tools that are easy to test, maintain, and scale.

[![PyPI - Version](https://img.shields.io/pypi/v/bijux-cli.svg)](https://pypi.org/project/bijux-cli/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/bijux-cli.svg)](https://pypi.org/project/bijux-cli/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/bijux/bijux-cli/main/LICENSES/MIT.txt)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-brightgreen)](https://bijux.github.io/bijux-cli/)
[![CI Status](https://github.com/bijux/bijux-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-cli/actions)

> **At a glance:** Plugin-driven • deterministic flags • DI for testability • REPL • structured JSON/YAML  
> **Quality:** **2,600+ tests** across layers with **98%+ coverage**. Multi-version CI. Docs build enforced. No telemetry.

---

## Table of Contents

* [Why Bijux CLI?](#why-bijux-cli)
* [Try It in 20 Seconds](#try-it-in-20-seconds)
* [Key Features](#key-features)
* [Installation](#installation)
* [Quick Start](#quick-start)
* [Plugins in 60 Seconds](#plugins-in-60-seconds)
* [Structured Output](#structured-output)
* [Developer Introspection](#developer-introspection)
* [Global Flags: Strict Precedence](#global-flags-strict-precedence)
* [Built-in Commands](#built-in-commands)
* [When to Use (and Not Use)](#when-to-use-and-not-use)
* [Shell Completion](#shell-completion)
* [Configuration & Paths](#configuration--paths)
* [Tests & Quality](#tests--quality)
* [Project Tree](#project-tree)
* [Roadmap](#roadmap)
* [Docs & Resources](#docs--resources)
* [Contributing](#contributing)
* [Acknowledgments](#acknowledgments)
* [License](#license)

[Back to top](#top)

---

<a id="why-bijux-cli"></a>
## Why Bijux CLI?

Click and Typer excel at simple tools. Bijux emphasizes **predictability and modularity** for complex ones:

* **Deterministic flags** for reliable CI/scripting.
* **Dependency Injection kernel** for testable, decoupled services.
* **First-class plugins** to extend without touching the core.
* **Interactive REPL** for exploration and debugging.

[Back to top](#top)

---

<a id="try-it-in-20-seconds"></a>
## Try It in 20 Seconds

```bash
pipx install bijux-cli  # Or: pip install bijux-cli
bijux --version
bijux doctor
bijux status -f json --no-pretty
```

[Back to top](#top)

---

<a id="key-features"></a>

## Key Features

* **Plugin-Driven Extensibility** — Scaffold, install, validate; plugins become top-level commands.
* **Deterministic Behavior** ⚖ — Strict flag precedence (see ADR-0002).
* **DI Kernel** — Decouple internals; inspect graphs for debugging/tests.
* **REPL Shell** — Persistent session with history; great for exploration/demos.
* **Structured Output** — JSON/YAML (+ pretty/compact, verbosity, consistent errors).
* **Diagnostics** — Built-in `doctor`, `audit`, `docs` for workflows.
* **Shell Completion** — Bash, Zsh, Fish, PowerShell support.

[Back to top](#top)

---

<a id="installation"></a>

## Installation

Requires **Python 3.11+**.

```bash
# Isolated install (recommended)
pipx install bijux-cli

# Standard
pip install bijux-cli
```

Upgrade: `pipx upgrade bijux-cli` or `pip install --upgrade bijux-cli`.

[Back to top](#top)

---

<a id="quick-start"></a>

## Quick Start

```bash
# Discover commands/flags
bijux --help

# Health check
bijux doctor

# REPL mode
bijux
bijux> help
bijux> status
bijux> exit
```

[Back to top](#top)

---

<a id="plugins-in-60-seconds"></a>

## Plugins in 60 Seconds

```bash
# Scaffold from a real template (local dir or Git URL), then install
# Option A: local template (example uses repo's cookiecutter template)
bijux plugins scaffold my_plugin --template ./plugin_template --force

# Option B: cookiecutter-compatible Git URL
# bijux plugins scaffold my_plugin --template https://github.com/bijux/bijux-plugin-template.git --force

# Install & explore
bijux plugins install ./my_plugin --force
bijux plugins list
bijux my_plugin --help

# Validate & remove
bijux plugins check my_plugin
bijux plugins uninstall my_plugin
```

Plugins dynamically add top-level commands.

[Back to top](#top)

---

<a id="structured-output"></a>

## Structured Output

For automation:

```bash
# Compact JSON
bijux status -f json --no-pretty | jq

# Pretty YAML
bijux status -f yaml --pretty
```

[Back to top](#top)

---

<a id="developer-introspection"></a>

## Developer Introspection

```bash
# DI graph
bijux dev di -f json

# Loaded plugins
bijux dev list-plugins
```

[Back to top](#top)

---

<a id="global-flags-strict-precedence"></a>

## Global Flags: Strict Precedence

Fixed ordering eliminates ambiguity.

| Priority | Flag                        | Effect                                                    |
| -------: | --------------------------- | --------------------------------------------------------- |
|        1 | `-h`, `--help`              | Immediate exit (code 0) with usage; ignores all.          |
|        2 | `-q`, `--quiet`             | Suppress stdout/stderr; preserves exit code.              |
|        3 | `-d`, `--debug`             | Full diagnostics; implies `--verbose`, forces `--pretty`. |
|        4 | `-f, --format <json\|yaml>` | Structured output; invalid → code 2.                      |
|        5 | `--pretty` / `--no-pretty`  | Indentation toggle (default: `--pretty`).                 |
|        6 | `-v`, `--verbose`           | Runtime metadata; implied by `--debug`.                   |

Rationale: [ADR-0002](https://bijux.github.io/bijux-cli/ADR/0002-global-flags-precedence)

[Back to top](#top)

---

<a id="built-in-commands"></a>

## Built-in Commands

| Command   | Description                 | Example                           |
| --------- | --------------------------- | --------------------------------- |
| `doctor`  | Environment diagnostics     | `bijux doctor`                    |
| `status`  | CLI snapshot                | `bijux status -f json`            |
| `repl`    | Interactive shell           | `bijux repl`                      |
| `plugins` | Manage plugins              | `bijux plugins list`              |
| `config`  | Key-value settings          | `bijux config set core_timeout=5` |
| `history` | REPL history                | `bijux history --limit 10`        |
| `audit`   | Security checks             | `bijux audit --dry-run`           |
| `docs`    | Generate specs/docs         | `bijux docs --out spec.json`      |
| `dev`     | Introspection (DI, plugins) | `bijux dev di`                    |
| `sleep`   | Pause                       | `bijux sleep -s 5`                |
| `version` | Version info                | `bijux version`                   |

[Back to top](#top)

---

<a id="when-to-use-and-not-use"></a>

## When to Use (and Not Use)

**Use if you need:**

* Plugins for extensibility.
* Deterministic flags for CI/scripts. [ADR-0002](https://bijux.github.io/bijux-cli/ADR/0002-global-flags-precedence)
* REPL for interactive workflows.
* DI for modular, testable design.

**Overkill if:**

* You’re building a tiny one-off script (Click/Typer may be simpler).
* You don’t need plugins/DI.

[Back to top](#top)

---

<a id="shell-completion"></a>

## Shell Completion

```bash
# Install (writes to your shell’s completion dir)
bijux --install-completion

# Or print the script for manual setup
bijux --show-completion
```

*Zsh tip:* Ensure `compinit` runs and your `fpath` includes the completion directory.

[Back to top](#top)

---

<a id="configuration--paths"></a>

## Configuration & Paths

Precedence: **flags > env > config > defaults**.

* Config: `~/.bijux/.env` (`BIJUXCLI_CONFIG`)
* History: `~/.bijux/.history` (`BIJUXCLI_HISTORY_FILE`)
* Plugins: `~/.bijux/.plugins` (`BIJUXCLI_PLUGINS_DIR`)

Example:

```bash
export BIJUXCLI_PLUGINS_DIR=./custom-plugins
```

[Back to top](#top)

---

<a id="tests--quality"></a>

## Tests & Quality

* **Depth:** 2,600+ tests across unit, integration, functional, and E2E layers.
* **Coverage:** **98%+** code coverage (measured via `pytest-cov` in CI).
* **Determinism:** CI runs the full suite on multiple Python versions (3.11+).
* **Artifacts:** JSON/YAML fixtures validate structured outputs; E2E simulates real usage (REPL, plugins, DI).
* **Docs:** Read the full testing guide → **[TESTS.md](TESTS.md)**.

Quick commands:

```bash
make test         # all tests
make test-unit    # unit tests only
make test-e2e     # end-to-end tests only
```

[Back to top](#top)

---

<a id="project-tree"></a>

## Project Tree

A guided map of the repository (what lives where, and why).
See **[PROJECT\_TREE.md](PROJECT_TREE.md)** for the full breakdown.

Quick glance:

```
api/            # OpenAPI schemas
config/         # Lint/type/security configs
docs/           # MkDocs site (Material)
makefiles/      # Task modules (docs, test, lint, etc.)
plugin_template/# Cookiecutter-ready plugin scaffold
scripts/        # Helper scripts (hooks, docs generation)
src/bijux_cli/  # CLI + library implementation
tests/          # unit / integration / functional / e2e
```

[Back to top](#top)

---

<a id="roadmap"></a>

## Roadmap

* **v0.2** — Async command support, richer plugin registry.
* **v1.0** — Community plugin marketplace, benchmarks vs. alternatives.

Track progress and suggest features via [Issues](https://github.com/bijux/bijux-cli/issues).

[Back to top](#top)

---

<a id="docs--resources"></a>

## Docs & Resources

* **Site**: [https://bijux.github.io/bijux-cli/](https://bijux.github.io/bijux-cli/)
* **Changelog**: [https://github.com/bijux/bijux-cli/blob/main/CHANGELOG.md](https://github.com/bijux/bijux-cli/blob/main/CHANGELOG.md)
* **Repository**: [https://github.com/bijux/bijux-cli](https://github.com/bijux/bijux-cli)
* **Issues**: [https://github.com/bijux/bijux-cli/issues](https://github.com/bijux/bijux-cli/issues)
* **Security** (private reports): [https://github.com/bijux/bijux-cli/security/advisories/new](https://github.com/bijux/bijux-cli/security/advisories/new)
* **Tests:** See **[TESTS.md](TESTS.md)**
* **Project Tree:** See **[PROJECT\_TREE.md](PROJECT_TREE.md)**

*When filing issues, include `--debug` output where possible.*

[Back to top](#top)

---

<a id="contributing"></a>

## Contributing

Welcome! See **[CONTRIBUTING.md](https://github.com/bijux/bijux-cli/blob/main/CONTRIBUTING.md)** for setup, style, and tests. We label **good first issue** to help you get started.

[Back to top](#top)

---

<a id="acknowledgments"></a>

## Acknowledgments

* Built on [Typer](https://typer.tiangolo.com/) (CLI), [FastAPI](https://fastapi.tiangolo.com/) (HTTP API), and [Injector](https://github.com/python-injector/injector) (DI).
* Inspired by Click, Typer, and Cobra.
* Thanks to early contributors and testers!

[Back to top](#top)

---

<a id="license"></a>

## License

MIT — see **[LICENSES/MIT.txt](https://raw.githubusercontent.com/bijux/bijux-cli/main/LICENSES/MIT.txt)**.
© 2025 Bijan Mousavi.

[Back to top](#top)
