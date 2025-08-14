# Bijux CLI — User Guide
<a id="top"></a>

A practical reference for installation, global flags, commands, configuration, and behavior. For a project overview and motivation, see the README on GitHub: [https://github.com/bijux/bijux-cli](https://github.com/bijux/bijux-cli)

---

## Table of Contents

* [Installation](#installation)
* [Quick Start](#quick-start)
* [Shell Completion](#shell-completion)
* [Global Flags – Precedence](#global-flags--precedence)
* [Command Reference](#command-reference)

  * [`config` — Settings](#config--settings)
  * [`plugins` — Plugin Management](#plugins--plugin-management)
  * [`history` — REPL History](#history--repl-history)
  * [`dev` — Developer Tools](#dev--developer-tools)
* [Built-in Commands (Index)](#built-in-commands-index)
* [Advanced Usage Patterns](#advanced-usage-patterns)
* [Configuration](#configuration)
* [End-to-End Workflows](#end-to-end-workflows)

  * [Core Workflow (no plugins)](#core-workflow-no-plugins)
  * [Plugin Workflow (requires a real template)](#plugin-workflow-requires-a-real-template)
* [Structured Error Model](#structured-error-model)
* [Exit Codes](#exit-codes)
* [Troubleshooting & FAQs](#troubleshooting--faqs)

[Back to top](#top)

---

<a id="installation"></a>

## Installation

Requires **Python 3.11+**.

### pipx (recommended)

```bash
pipx install bijux-cli
pipx ensurepath       # first-time pipx users
# later
pipx upgrade bijux-cli
```

### pip

```bash
python -m pip install -U bijux-cli
```

> Tip: use a virtual environment when installing with plain `pip`.

[Back to top](#top)

---

<a id="quick-start"></a>

## Quick Start

```bash
bijux --help          # commands and global flags
bijux --version       # version/sanity
bijux doctor          # environment diagnostics

# REPL
bijux
bijux> help
bijux> status -f json   # {"version":"<current>", ...}
bijux> exit
```

[Back to top](#top)

---

<a id="shell-completion"></a>

## Shell Completion

```bash
bijux --install-completion   # install for your current shell
bijux --show-completion      # print the script for manual setup
```

Shell notes:

* **Bash** (current session): `eval "$(bijux --show-completion)"`
* **Zsh** (persistent): add to `~/.zshrc`:

  * `fpath+=("$HOME/.zfunc")`
  * `autoload -U compinit && compinit`
  * then run `bijux --install-completion`
* **Fish / PowerShell**: run `--install-completion`

> Restart your shell after installing. For Zsh, ensure `compinit` runs and your `fpath` includes the completions directory.

[Back to top](#top)

---

<a id="global-flags--precedence"></a>

## Global Flags – Precedence

Flags are evaluated in strict order; higher priority short-circuits lower ones.

| Priority | Flag(s)                         | Behavior                                                     |
| -------: | ------------------------------- | ------------------------------------------------------------ |
|        1 | `-h`, `--help`                  | Exit 0 with usage; ignore all other flags.                   |
|        2 | `-q`, `--quiet`                 | Suppress stdout/stderr; exit code still reflects result.     |
|        3 | `-d`, `--debug`                 | Full diagnostics; implies `--verbose` and forces `--pretty`. |
|        4 | `-f`, `--format` `<json\|yaml>` | Structured output; invalid value → exit code 2.              |
|        5 | `--pretty` / `--no-pretty`      | Indentation control (default: `--pretty`).                   |
|        6 | `-v`, `--verbose`               | Include runtime metadata; implied by `--debug`.              |

Details: ADR-0002 (Global Flags Precedence) — [https://bijux.github.io/bijux-cli/ADR/0002-global-flags-precedence](https://bijux.github.io/bijux-cli/ADR/0002-global-flags-precedence)

> When `--format` is set, **errors** are emitted in that format to **stderr** (unless `--quiet`).

[Back to top](#top)

---

<a id="command-reference"></a>

## Command Reference

<a id="config--settings"></a>

### `config` — Settings

Dotenv-style key/value settings. Keys must be alphanumeric or underscore.

* `list` → `{"items":[{"key":"...","value":"..."}]}`

  ```bash
  bijux config list -f json --no-pretty
  ```
* `get <key>` → `{"value":"..."}`

  ```bash
  bijux config get core_timeout
  ```
* `set <key=value>` / `unset <key>`

  ```bash
  bijux config set core_timeout=30
  bijux config unset core_timeout
  ```
* `export <path>` (supports `-f json|yaml`)

  ```bash
  bijux config export ./settings.env
  bijux config export ./settings.json -f json
  ```
* `load <path>` (dotenv), `reload`, `clear`

> Tip: Use `-f json --no-pretty` for machine-readable output in scripts.

[Back to top](#top)

---

<a id="plugins--plugin-management"></a>

### `plugins` — Plugin Management

Default install dir: `~/.bijux/.plugins` (override via `BIJUXCLI_PLUGINS_DIR`).

* `list` → `{"plugins":["...", ...]}`

  ```bash
  bijux plugins list
  ```
* `info <name|path>`, `check <name|path>`, `uninstall <name>`
* `install <path>` — install a plugin directory (use `--force` to overwrite)

  ```bash
  bijux plugins install ./path/to/my_plugin --force
  ```
* `scaffold <name> --template <path-or-git-url>` — create a plugin from a template

  ```bash
  # Requires a real template (local dir or cookiecutter-compatible Git URL)
  bijux plugins scaffold my_plugin --template ./templates/bijux-plugin --force
  # or
  bijux plugins scaffold my_plugin --template https://github.com/bijux/bijux-plugin-template.git --force
  ```

> `scaffold` **requires** `--template`. Without it you’ll get `no_template`.
> `--force` overwrites files in the destination.

[Back to top](#top)

---

<a id="history--repl-history"></a>

### `history` — REPL History

* `list` (supports `--limit`, `--group-by`, `--filter`, `--sort`) → `{"entries":[...]}`

  ```bash
  bijux history --limit 10 -f json --no-pretty
  ```
* `--export <path>`, `--import <path>`, `clear`

[Back to top](#top)

---

<a id="dev--developer-tools"></a>

### `dev` — Developer Tools

* `di` → `{"factories":[...],"services":[...]}`

  ```bash
  bijux dev di -f json
  ```
* `list-plugins` — diagnostic list of discovered plugins

[Back to top](#top)

---

<a id="built-in-commands-index"></a>

## Built-in Commands (Index)

| Command   | Purpose                    | Example (sample output)                                 |
| --------- | -------------------------- | ------------------------------------------------------- |
| `audit`   | Security/compliance checks | `bijux audit --dry-run` → `{"issues":[...]}`            |
| `docs`    | Generate specs/docs        | `bijux docs --out spec.json` → writes file              |
| `doctor`  | Health diagnostics         | `bijux doctor` → summary or detailed findings           |
| `memory`  | Key-value store            | `bijux memory set key=val` → `{"status":"set"}`         |
| `repl`    | Interactive shell          | `bijux repl` → interactive `bijux>` prompt              |
| `sleep`   | Pause execution            | `bijux sleep -s 5` → pauses for 5 seconds               |
| `status`  | CLI status snapshot        | `bijux status -f json` → `{"version":"<current>", ...}` |
| `version` | Version info               | `bijux version` → `<current>`                           |

Plugins appear as additional top-level commands after install.

[Back to top](#top)

---

<a id="advanced-usage-patterns"></a>

## Advanced Usage Patterns

* Batch config apply:

  ```bash
  bijux config set core_timeout=30 && bijux config reload
  ```
* Check all installed plugins:

  ```bash
  bijux plugins list -f json --no-pretty \
    | jq -r '.plugins[]' \
    | xargs -I {} bijux plugins check {}
  ```
* Diagnostics pipeline:

  ```bash
  bijux doctor --debug > diag.log
  bijux status -d >> diag.log
  ```

> Combine with `--quiet` in CI to suppress output while preserving exit codes.

[Back to top](#top)

---

<a id="configuration"></a>

## Configuration

**Default paths** (overridable via environment variables):

* Config file: `~/.bijux/.env` (`BIJUXCLI_CONFIG`)
* History file: `~/.bijux/.history` (`BIJUXCLI_HISTORY_FILE`)
* Plugins dir: `~/.bijux/.plugins` (`BIJUXCLI_PLUGINS_DIR`)

Override example (shell profile):

```bash
export BIJUXCLI_PLUGINS_DIR=./plugins
```

**Resolution precedence:**

1. CLI flags → 2) Environment variables → 3) Config file → 4) Defaults

[Back to top](#top)

---

<a id="end-to-end-workflows"></a>

## End-to-End Workflows

<a id="core-workflow-no-plugins"></a>

### Core Workflow (no plugins)

This flow works on any install; it does not assume a template or extra files.

```bash
# Fresh artifacts directory
rm -rf artifacts && mkdir -p artifacts

# Version and health
bijux --version
bijux doctor

# Config operations
bijux config set core_timeout=30
bijux config get core_timeout                 # {"value":"30"}
bijux config list -f json --no-pretty > artifacts/config.json
bijux config export artifacts/settings.env
bijux config export artifacts/settings.json -f json

# REPL creates history
bijux repl <<'EOF'
version
help
exit
EOF

# History operations
bijux history --limit 5 -f json --no-pretty > artifacts/history.json
bijux history --export artifacts/history-full.json
bijux history --import artifacts/history-full.json

# Cleanup
bijux history clear
bijux config clear
```

<a id="plugin-workflow-requires-a-real-template"></a>

### Plugin Workflow (requires a real template)

Choose **one**:

* **A) Local template directory** (e.g., `./templates/bijux-plugin`)
* **B) Cookiecutter-compatible Git URL** (e.g., https://github.com/bijux/bijux-plugin-template.git )

```bash
# Start clean
bijux plugins uninstall my_plugin || true
rm -rf tmp && mkdir -p tmp

# Scaffold (requires a real template path or Git URL)
# Option A: local template directory
bijux plugins scaffold my_plugin --template ./templates/bijux-plugin --force

# Option B: cookiecutter-compatible Git URL
# bijux plugins scaffold my_plugin --template https://github.com/bijux/bijux-plugin-template.git --force

# Install the newly scaffolded plugin
bijux plugins install ./my_plugin --force

# Verify & validate
bijux plugins list                     # {"plugins":["my_plugin", ...]}
bijux plugins info my_plugin
bijux plugins check my_plugin

# Uninstall when done
bijux plugins uninstall my_plugin || true
```

> If you do not provide a real template, `scaffold` will fail with `no_template`, and subsequent `install/info/check` will also fail.

[Back to top](#top)

---

<a id="structured-error-model"></a>

## Structured Error Model

With `--format`, errors are structured and written to **stderr** (unless `--quiet`):

```json
{
  "error": "message",
  "code": 2,
  "failure": "machine_readable_reason",
  "command": "subcommand path",
  "fmt": "json"
}
```

(YAML is emitted when `-f yaml` is used.)

[Back to top](#top)

---

<a id="exit-codes"></a>

## Exit Codes

| Code | Meaning                |
| ---: | ---------------------- |
|    0 | Success                |
|    1 | General/internal error |
|    2 | Usage/invalid argument |
|    3 | Encoding/hygiene error |

Commands may define additional non-conflicting codes.

[Back to top](#top)

---

<a id="troubleshooting--faqs"></a>

## Troubleshooting & FAQs

* Start with `bijux doctor`.
* Need more detail? Use `--verbose` or `--debug` (adds pretty printing and diagnostics).
* Scripting? Prefer `-f json --no-pretty` and read from **stdout**; errors go to **stderr**.
* Completion not working? Re-run `--install-completion` and restart the shell; ensure Zsh `compinit` and `fpath` are correct.
* Permission denied? Ensure paths are writable; avoid `sudo` unless absolutely required.
* Plugin errors?

  * `no_template`: pass a real `--template` (path or Git URL) to `plugins scaffold`.
  * `not_found` / `not_installed`: confirm plugin name; check `bijux plugins list`.
  * Use `bijux plugins check <name>` after installing.
* Bug reports: include `--debug` output, version (`bijux --version`), OS, and repro steps.

[Back to top](#top)
