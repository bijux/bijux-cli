# Bijux CLI

<a id="top"></a>

**Bijux CLI is a Rust-owned command runtime with plugin routing, structured output, and a Python compatibility bridge.**

This repository currently ships three connected surfaces:

* `bijux`: the end-user runtime binary,
* `bijux-cli`: the Python distribution and Rust crate that package that runtime,
* `bijux-dev-cli`: the workspace-only maintainer control plane.

The public promise today is a deterministic CLI with explicit plugin lifecycle handling, stable machine-readable output, and shared CLI/REPL behavior.
Git checkout builds derive runtime identity from the latest real `v*` tag in Git until a newer release tag exists, so a bumped workspace manifest alone does not make the checkout claim a published release.

[![PyPI - Version](https://img.shields.io/pypi/v/bijux-cli.svg)](https://pypi.org/project/bijux-cli/)
[![crates.io](https://img.shields.io/crates/v/bijux-cli.svg)](https://crates.io/crates/bijux-cli)
[![Rust Docs](https://img.shields.io/badge/docs-Rust%20Docs-blue)](https://docs.rs/bijux-cli/latest/bijux_cli/)
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://pypi.org/project/bijux-cli/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-brightgreen)](https://bijux.io/bijux-core/bijux-cli/)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-core/actions)

Python package: [PyPI](https://pypi.org/project/bijux-cli/)
Rust crate: [crates.io](https://crates.io/crates/bijux-cli)
Rust API docs: [docs.rs](https://docs.rs/bijux-cli/latest/bijux_cli/)
Rust source docs: [docs.rs source](https://docs.rs/crate/bijux-cli/latest/source/)

> **At a glance**
> Rust runtime · Python bridge · Routed plugins · REPL · JSON/YAML output
> **Quality**
> Quality status is checked from live tests and maintainer commands.
> `artifacts/` is disposable local output and is not part of the repo contract.
> The repository may be ahead of the latest published release between version tags.

---

## Table of Contents

* [Why Bijux CLI?](#why-bijux-cli)
* [What Ships Today](#what-ships-today)
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
* [Release Line & Stability](#release-line--stability)
* [Docs & Resources](#docs--resources)
* [Contributing](#contributing)
* [License](#license)

---

## Why Bijux CLI?

Bijux is for command surfaces where:

* global flags must behave consistently in CI and automation,
* structured output matters as much as human-readable output,
* plugins must be installable, inspectable, and removable without patching the core binary,
* REPL and CLI behavior should not diverge,
* and maintainers need evidence for release claims instead of guesswork.

---

## What Ships Today

The repository is strongest when it stays concrete about what is already real:

* a Rust runtime binary named `bijux`,
* top-level runtime commands such as `status`, `audit`, `docs`, `doctor`, `install`, and `plugins`,
* a canonical `cli` namespace for runtime inspection and compatibility routes,
* routed plugins with install, inspect, check, explain, enable, disable, and uninstall flows,
* a Python bridge package that delegates to the Rust runtime,
* and a separate maintainer binary, `bijux-dev-cli`, for workspace evidence and control-plane reports.

This README intentionally describes those shipped surfaces, not broader framework ambition.

---

## Key Features

### Deterministic Runtime Surface

Global flags follow strict precedence, root commands keep stable names, and the
REPL mirrors the same command semantics instead of inventing a separate mode.

### Canonical Runtime Namespace

The root command surface is the main end-user entrypoint. The `cli` namespace
is the canonical runtime-management surface for paths, self-tests, completion,
plugin lifecycle commands, and compatibility inspection.

### Routed Plugins

Plugins are scaffolded as local projects, validated before install, mounted
into the runtime namespace after install, and kept visible through list,
inspect, explain, check, and doctor workflows.

### Interactive REPL

Explore and debug using a persistent shell:

* identical semantics to CLI execution,
* history and introspection built in.

### Structured Output and Diagnostics

Every command can emit:

* JSON or YAML,
* pretty or compact,
* consistent error envelopes suitable for automation.

Commands such as `doctor`, `status`, `audit`, and `docs` are read-only
diagnostic surfaces. They report environment and runtime state; they are not
deployment orchestration or workflow engines by themselves.

### Python Compatibility Surface

The Python package exists to package and bridge the Rust runtime. It does not
replace runtime ownership or reintroduce a separate Python-first command graph.

---

## Installation

Use one install channel at a time.

Published versions come from PyPI and crates.io. Repository checkouts may carry
release-preparation manifests between tags, so install from a release channel
for a stable published build and use `cargo run` from a checkout when you want
the current source tree. Checkout builds still report the latest real release
tag line until a newer `v*` tag exists.

Supported install channels today:

```bash
# Cargo
cargo install --locked bijux-cli

# Pipx
pipx install bijux-cli

# Pip
python3 -m pip install --upgrade bijux-cli
```

Quick verification:

```bash
bijux version
bijux cli paths
bijux status
bijux doctor
```

`bijux doctor` is the install-health check. Treat `warning` and `degraded`
results as part of the install outcome rather than a postscript. `bijux cli
paths` makes the active binary and state locations explicit when you need to
confirm which install surface is actually in use.

From a workspace checkout, run the branch version directly with:

```bash
cargo run -q -p bijux-cli --bin bijux -- version
```

---

## Platform Support

* **Supported host contract today**: Linux, macOS
* **Not supported today**: Windows

The Rust runtime now includes some Windows-aware code paths such as `.exe`
discovery, state-path resolution, and install-path diagnostics. That is real
progress, but it is not enough to advertise Windows as an end-to-end supported
host.

The current blockers are still product-level, not just packaging-level:

* the built-in Rust plugin scaffold emits a POSIX shell entrypoint,
* completion target paths and install guidance are Linux/macOS-oriented,
* and the documented support and release contract is still written around
  Linux/macOS hosts.

`pwsh` completion output is supported on Linux and macOS hosts where PowerShell
is installed. That does not imply general Windows runtime support.

---

## Plugins in 60 Seconds

```bash
# Scaffold a Python plugin
bijux plugins scaffold python my-plugin --path ./my-plugin --force

# Scaffold a Cargo-backed Rust plugin
bijux plugins scaffold rust my-rust-plugin --path ./my-rust-plugin --force

# First Rust execution builds the local debug binary in the plugin's own target/ tree if needed
./my-rust-plugin/plugin-entrypoint --help

# Install and explore
bijux plugins install ./my-plugin
bijux plugins list
bijux cli plugins list
bijux plugins inspect my-plugin
bijux my-plugin --help
# Routed aliases follow the same install contract
# if your manifest declares aliases: ["my-shortcut"]
# bijux my-shortcut --help
bijux plugins explain my-plugin

# Validate and remove
bijux plugins check my-plugin
bijux plugins uninstall my-plugin
```

Plugins provide a managed install, inspection, diagnostics, and routed runtime
surface without baking plugin code into the core runtime binary. `plugins install`
accepts either a plugin directory root or an explicit `plugin.manifest.json`
path. Python-backed plugins require Python 3.11 or newer on `PATH`. The
top-level `plugins` commands are the ordinary end-user surface; `cli plugins`
is the canonical compatibility namespace for the same lifecycle area.

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

Format flags are shared across the runtime surface. Successful payloads are
stable per command, and error payloads stay machine-readable instead of falling
back to ad hoc prose.

---

## Developer Introspection

Maintainer diagnostics run through `bijux-dev-cli` from a workspace build or
checkout. They are not part of the ordinary end-user command surface. Runtime
install aliases such as `bijux install dev-cli --dry-run` resolve package names,
but they do not make maintainer commands part of the `bijux` runtime namespace:

```bash
cargo run -q -p bijux-dev-cli --bin bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev-cli --bin bijux-dev-cli -- parity --format json --no-pretty
cargo run -q -p bijux-dev-cli --bin bijux-dev-cli -- state-doctor --format text
```

---

## Global Flags & Strict Precedence

Flags short-circuit in a fixed order.
Once a higher-priority flag applies, lower-priority inputs are ignored.
The table below describes precedence-sensitive flags, not every available
global switch.

| Priority | Flag                         | Effect                                 |
| -------: | ---------------------------- | -------------------------------------- |
|        1 | `-h`, `--help`               | Immediate exit with usage              |
|        2 | `-q`, `--quiet`              | Suppress stdout/stderr                 |
|        3 | `--log-level debug`          | Full diagnostics; forces pretty output |
|        4 | `-f`, `--format json / yaml` | Structured output                      |
|        5 | `--pretty / --no-pretty`     | Formatting toggle                      |
|        6 | `--log-level <level>`        | Logging threshold                      |

See the execution model in the [Introduction docs](https://bijux.io/bijux-core/bijux-cli/).

---

## Built-in Commands

| Surface | Purpose |
| ------- | ------- |
| `status`, `audit`, `docs`, `doctor`, `version` | Read-only runtime and environment diagnostics |
| `install`, `completion` | Installation guidance and shell integration |
| `config`, `history`, `memory` | Runtime-owned state and compatibility paths |
| `plugins` | End-user plugin lifecycle commands |
| `repl` | Interactive runtime shell |
| `cli` | Canonical compatibility namespace for `status`, `paths`, `doctor`, `version`, `repl`, `completion`, `config`, `self-test`, and `plugins` |

---

## When to Use (and Not Use)

**Use Bijux if you need:**

* extensibility via plugins,
* deterministic behavior in CI,
* structured output and stable diagnostics,
* an interactive shell that follows the same runtime rules,
* and maintainable release evidence around the CLI surface.

**It may be overkill if:**

* you are writing a one-off script,
* you need a Windows-first runtime today,
* or you need a sandboxed plugin system with a permission model.

---

## Shell Completion

```bash
bijux completion --shell bash
bijux completion --shell fish --format json --no-pretty
```

Supports Bash, Zsh, Fish, and `pwsh` on supported Linux/macOS hosts.
Use `--shell` in automation so the output does not depend on environment
detection. Text output emits the selected shell script, while structured output
includes the active shell, how it was selected, supported shells, supported
platforms, a suggested target file, and the generated script. Windows is not
part of this completion contract today.

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

Suggested local baseline:

```bash
make bootstrap
make fmt
make lint
make test
make docs-check
```

Run `make all` when you want the full release-facing gate, including security
checks and Python package builds:

```bash
make all
```

For runtime-surface or release-facing changes, also check the maintainer views
that summarize the current repository contract:

```bash
cargo run -q -p bijux-dev-cli --bin bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev-cli --bin bijux-dev-cli -- parity --format json --no-pretty
cargo run -q -p bijux-dev-cli --bin bijux-dev-cli -- docs-audit --format json --no-pretty
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
contracts/      Repository contracts and registries
```

---

## Release Line & Stability

* The project is still pre-1.0, so compatibility tightening can still happen.
* Core CLI semantics such as flags, output formats, exit behavior, and runtime-owned state paths are the strongest stability surface today.
* Plugin lifecycle behavior is shipped and tested, but plugin metadata and loader internals can still evolve before v1.0.
* Published package versions and `v*` git tags define the public release line. Untagged checkout builds stay anchored to the latest real tag line even when the repository source tree has already moved ahead for the next release.
* Plugins are executable extension code, not a sandboxed trust boundary.

---

## Docs & Resources

* Documentation home: [https://bijux.io/bijux-core/bijux-cli/](https://bijux.io/bijux-core/bijux-cli/)
* Command reference: [https://bijux.io/bijux-core/bijux-cli/](https://bijux.io/bijux-core/bijux-cli/)
* Plugin guide: [https://bijux.io/bijux-core/bijux-cli/](https://bijux.io/bijux-core/bijux-cli/)
* Artifacts: [https://bijux.io/bijux-core/](https://bijux.io/bijux-core/)
* Repository: [https://github.com/bijux/bijux-core](https://github.com/bijux/bijux-core)

---

## Contributing

Contributions are welcome.
See [CONTRIBUTING.md](CONTRIBUTING.md).
When changing public behavior, update the README and reference docs so the
repository front page stays aligned with the shipped runtime.

---

## License

Apache-2.0.
© 2026 Bijan Mousavi.
