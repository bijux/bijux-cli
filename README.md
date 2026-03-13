# Bijux CLI

<a id="top"></a>

**Bijux CLI is a command interface with Rust runtime ownership and Python compatibility surfaces.**

It is designed for CLIs that grow over time and need stable command behavior.

Bijux focuses on deterministic flags, explicit plugin lifecycle behavior, structured output, and shared CLI/REPL command law.

[![PyPI - Version](https://img.shields.io/pypi/v/bijux-cli.svg)](https://pypi.org/project/bijux-cli/)
[![crates.io](https://img.shields.io/crates/v/bijux-cli.svg)](https://crates.io/crates/bijux-cli)
[![docs.rs](https://img.shields.io/docsrs/bijux-cli)](https://docs.rs/bijux-cli/latest/bijux_cli/)
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://pypi.org/project/bijux-cli/)
[![Typing: typed (PEP 561)](https://img.shields.io/badge/typing-typed-4F8CC9.svg)](https://peps.python.org/pep-0561/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://raw.githubusercontent.com/bijux/bijux-cli/main/LICENSE)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-brightgreen)](https://bijux.github.io/bijux-cli/)
[![CI Status](https://github.com/bijux/bijux-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-cli/actions)

Python package: [PyPI](https://pypi.org/project/bijux-cli/)
Rust crate: [crates.io](https://crates.io/crates/bijux-cli)
Rust API docs: [docs.rs](https://docs.rs/bijux-cli/latest/bijux_cli/)
Rust source docs: [docs.rs source](https://docs.rs/crate/bijux-cli/latest/source/)

> **At a glance**
> Plugin-driven · Deterministic flags · Dependency Injection · Sync + Async · REPL · JSON/YAML output
> **Quality**
> Quality status is checked from live tests and maintainer commands.
> `artifacts/` is disposable local output and is not part of the repo contract.

---

## Table of Contents

* [Why Bijux CLI?](#why-bijux-cli)
* [How to Think About Bijux](#how-to-think-about-bijux)
* [Key Features](#key-features)
* [Installation](#installation)
* [Platform Support](#platform-support)
* [Plugins in 60 Seconds](#plugins-in-60-seconds)
* [Plugin Non-Goals](#plugin-non-goals)
* [Structured Output](#structured-output)
* [Developer Introspection](#developer-introspection)
* [Global Flags & Strict Precedence](#global-flags--strict-precedence)
* [Built-in Commands](#built-in-commands)
* [When to Use (and Not Use)](#when-to-use-and-not-use)
* [Shell Completion](#shell-completion)
* [Configuration & Paths](#configuration--paths)
* [Tests & Quality](#tests--quality)
* [Project Tree](#project-tree)
* [Stability Notes](#stability-notes)
* [Roadmap](#roadmap)
* [Docs & Resources](#docs--resources)
* [Contributing](#contributing)
* [Acknowledgments](#acknowledgments)
* [License](#license)

---

## Why Bijux CLI?

Bijux is for CLIs where:

* global flags must behave consistently in CI and automation,
* commands may be synchronous or asynchronous,
* features and plugins are added incrementally,
* internal state must be observable and testable,
* and regressions must be caught early.

---

## How to Think About Bijux

A Bijux command flows through a **fixed, explicit pipeline**:

```
intent → policy resolution → (sync | async) execution → emission → exit
```

Key principles:

* **Flags never compete** — precedence is strict and deterministic.
* **Decisions are made once**, early in execution.
* **Services are injected**, never hidden behind globals.
* **Commands do not format output** — emission is centralized.
* **Async and sync commands share the same semantics**.
* **The REPL uses the exact same execution path as the CLI**.

If you reason about Bijux in these terms, the framework becomes predictable rather than magical.

---

## Key Features

### Deterministic Global Flags

Global flags follow **strict precedence**, eliminating ambiguity and unexpected behavior in scripts and CI pipelines.

### Unified Sync + Async Execution

Commands may be implemented as synchronous or `async` functions.
Bijux runs both through the same execution pipeline, guaranteeing identical behavior for:

* flag precedence,
* output formatting,
* logging,
* and exit codes.

Async and sync commands use one execution path.

### Dependency Injection (DI)

All services are explicit and injectable:

* no hidden globals,
* easy mocking,
* inspectable dependency graphs.

### First-Class Plugins

Plugins are treated as real system components:

* scaffolded from templates,
* validated before loading,
* dynamically exposed as top-level commands.

### Interactive REPL

Explore and debug using a persistent shell:

* identical semantics to CLI execution,
* history and introspection built in.

### Structured Output

Every command can emit:

* JSON or YAML,
* pretty or compact,
* consistent error envelopes suitable for automation.

### Built-in Diagnostics

Commands like `doctor`, `audit`, and `docs` help verify environments and workflows.

---

## Installation

Requires **Python ≥ 3.11** (3.11–3.13 tested).

```bash
# Recommended (isolated)
pipx install bijux-cli

# Standard
python3 -m pip install --upgrade bijux-cli
```

Upgrade with `pipx upgrade bijux-cli` or `python3 -m pip install --upgrade bijux-cli`.

Quick verification:

```bash
bijux --version
bijux doctor
bijux status --format json --no-pretty
```

---

## Platform Support

* **Supported**: Linux, macOS
* **Not supported**: Windows

Bijux relies on POSIX filesystem and process semantics.

---

## Plugins in 60 Seconds

```bash
# Scaffold a Python plugin from repository templates
bijux plugins scaffold python my-plugin --path ./my-plugin --force

# Scaffold a Rust-backed plugin from repository templates
bijux plugins scaffold rust my-rust-plugin --path ./my-rust-plugin --force

# Install and explore
bijux plugins install ./my-plugin/plugin.manifest.json
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins explain my-plugin

# Validate and remove
bijux plugins check my-plugin
bijux plugins uninstall my-plugin
```

Plugins provide a managed install, inspection, and diagnostics surface without
changing the core runtime command inventory.

---

## Plugin Non-Goals

Bijux plugins are **not sandboxed**.

There are:

* no security guarantees,
* no isolation,
* no permission model.

Only install plugins you trust.

---

## Structured Output

For automation and scripting:

```bash
# Compact JSON
bijux status -f json --no-pretty | jq

# Pretty YAML
bijux status -f yaml --pretty
```

---

## Developer Introspection

Maintainer diagnostics run through `bijux-dev-cli` from a workspace build or
checkout:

```bash
cargo run -q -p bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev-cli -- parity --format json --no-pretty
cargo run -q -p bijux-dev-cli -- state-doctor --text
```

---

## Global Flags & Strict Precedence

Flags short-circuit in a fixed order.
Once a higher-priority flag applies, lower-priority inputs are ignored.

| Priority | Flag                         | Effect                                 |
| -------: | ---------------------------- | -------------------------------------- |
|        1 | `-h`, `--help`               | Immediate exit with usage              |
|        2 | `-q`, `--quiet`              | Suppress stdout/stderr                 |
|        3 | `--log-level debug`          | Full diagnostics; forces pretty output |
|        4 | `-f`, `--format json / yaml` | Structured output                      |
|        5 | `--pretty / --no-pretty`     | Formatting toggle                      |
|        6 | `--log-level <level>`        | Logging threshold                      |

See the execution model in the [Introduction docs](https://bijux.github.io/bijux-cli/01-introduction/command-model/).

---

## Built-in Commands

| Command   | Purpose                 |
| --------- | ----------------------- |
| `doctor`  | Environment diagnostics |
| `status`  | CLI snapshot            |
| `repl`    | Interactive shell       |
| `plugins` | Manage plugins          |
| `config`  | Key-value settings      |
| `history` | REPL history            |
| `audit`   | Security checks         |
| `docs`    | Generate specs/docs     |
| `sleep`   | Pause execution         |
| `version` | Version info            |

---

## When to Use (and Not Use)

**Use Bijux if you need:**

* extensibility via plugins,
* deterministic behavior in CI,
* sync + async commands under one model,
* structured output,
* testable internals.

**It may be overkill if:**

* you are writing a one-off script,
* your CLI will never grow,
* plugins and DI provide no value.

---

## Shell Completion

```bash
bijux completion
bijux completion --format json
```

Supports Bash, Zsh, Fish, and PowerShell.

---

## Configuration & Paths

Precedence: **flags → env → config → defaults**

| Purpose | Path                | Env                     |
| ------- | ------------------- | ----------------------- |
| Config  | `~/.bijux/.env`     | `BIJUXCLI_CONFIG`       |
| History | `~/.bijux/.history` | `BIJUXCLI_HISTORY_FILE` |
| Plugins | `~/.bijux/.plugins` | `BIJUXCLI_PLUGINS_DIR`  |

---

## Tests & Quality

Bijux quality claims are checked through tests, CI gates, and live maintainer
commands.

Run locally:

```bash
make env
make test-py
make test-unit-py
make test-nightly-py
make fmt-rs lint-rs audit-rs test-all-rs
make build-py
make publish-py    # requires PYPI_API_TOKEN; set RELEASE_VERSION=x.y.z for a tagged release publish
make publish-rs    # dry-run by default; set RUST_PUBLISH_DRY_RUN=0 RELEASE_VERSION=x.y.z to publish a tagged release
```

Parity checks worth running:

```bash
python3 -m pytest crates/bijux-cli-python/tests/python/test_runtime_parity.py
BIJUX_ENABLE_STABLE_PYPI_PARITY=1 python3 -m pytest -m nightly crates/bijux-cli-python/tests/python/test_stable_release_compatibility.py
```

---

## Project Tree

```text
configs/        Lint/type/security configs
docs/           Documentation (MkDocs)
makes/          Make modules
templates/      Plugin templates (plugins-py, plugins-rs)
crates/         Rust workspace crates (runtime, maintainer CLI, python bridge)
crates/*/tests/ Crate-local test suites
```

---

## Stability Notes

* Core CLI semantics (flags, precedence, exit behavior) are stable.
* The async execution model is stable and supported.
* Plugin metadata and loader internals may evolve before v1.0.
* Breaking changes, when unavoidable, will be documented clearly.

---

## Roadmap

Roadmap priorities come from open tests, maintainer review, and current command
surfaces.

---

## Docs & Resources

* Documentation: [https://bijux.github.io/bijux-cli/](https://bijux.github.io/bijux-cli/)
* Artifacts: [https://bijux.github.io/bijux-cli/artifacts/](https://bijux.github.io/bijux-cli/artifacts/)
* Repository: [https://github.com/bijux/bijux-cli](https://github.com/bijux/bijux-cli)

---

## Contributing

Contributions are welcome.
See [CONTRIBUTING.md](https://github.com/bijux/bijux-cli/blob/main/CONTRIBUTING.md).

---

## Acknowledgments

Built on Typer, FastAPI, and Injector.  
Inspired by Click, Typer, and Cobra.  

---

## License

Apache-2.0.
© 2025 Bijan Mousavi.
