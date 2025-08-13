# Bijux CLI — User Guide

A concise, production-ready reference for **commands**, **global flags**, **configuration**, and **operational behavior**.

---

## Installation

**Requires:** Python 3.11+

### Using pip

```bash
python -m pip install -U bijux-cli
```

### Using pipx (recommended for isolation)

```bash
pipx install bijux-cli
# later
pipx upgrade bijux-cli
# first-time pipx users:
pipx ensurepath
```

---

## Quick Start

```bash
# See all commands and global flags
bijux --help

# Version / sanity check
bijux --version

# Environment health check
bijux doctor

# Launch interactive REPL
bijux
bijux> help
bijux> exit
```

---

## Shell Completion

Enable tab-completion (no sudo required).

```bash
# Install completion for your shell
bijux --install-completion

# Or print the script (for manual setup)
bijux --show-completion
```

**One-liners by shell**

* **Bash (current session)**

  ```bash
  eval "$(bijux --show-completion)"
  ```

* **Zsh (persist)**

  ```bash
  echo 'fpath+=("$HOME/.zfunc")' >> ~/.zshrc
  echo 'autoload -U compinit && compinit' >> ~/.zshrc
  bijux --install-completion
  ```

* **Fish**

  ```bash
  bijux --install-completion
  ```

* **PowerShell**

  ```powershell
  bijux --install-completion
  ```

> After installing, restart your shell. For zsh, ensure `compinit` runs and your `fpath` includes completions directory.

---

## Global Flags — Precedence Rules

Flags apply to every command and are evaluated in strict priority order. Higher-priority flags override or short-circuit lower ones.

| Priority | Flag(s)                     | Behavior                                                                  |
|---------:|-----------------------------|---------------------------------------------------------------------------|
|        1 | `-h`, `--help`              | Exit with code `0` immediately; ignore all other flags.                   |
|        2 | `-q`, `--quiet`             | Suppress **both stdout and stderr**; exit code still reflects the result. |
|        3 | `-d`, `--debug`             | Full diagnostics; implies `--verbose` and forces `--pretty`.              |
|        4 | `-f, --format <json\|yaml>` | Structured output format; invalid value → exit code `2`.                  |
|        5 | `--pretty` / `--no-pretty`  | Control indentation (default: `--pretty`).                                |
|        6 | `-v`, `--verbose`           | Add runtime metadata; implied by `--debug`.                               |

For details, see **ADR-0002: Global Flags Precedence**.

**Notes**

* When `--format` is set, **errors** are emitted in that format to **stderr** (unless `--quiet`).
* `--debug` implies `--verbose` and forces pretty printing.

---

## Command Reference

### `config` — Manage CLI Configuration

Key-value settings stored in a dotenv-style file. Keys must use **alphanumeric characters and underscores only** (no dots).

* **list** — `{"items":[{"key":"..."}, ...]}`

  ```bash
  bijux config list
  ```

* **get \<key>** — `{"value":"..."}`

  ```bash
  bijux config get core_timeout
  ```

* **set \<key=value>**

  ```bash
  bijux config set core_timeout=30
  ```

* **unset \<key>**

  ```bash
  bijux config unset core_timeout
  ```

* **export \<path>** (supports `--format json|yaml`)

  ```bash
  bijux config export ./settings.env
  bijux config export ./settings.json --format json
  ```

* **load \<path>** (dotenv format)

  ```bash
  bijux config load ./settings.env
  ```

* **reload**

  ```bash
  bijux config reload
  ```

* **clear**

  ```bash
  bijux config clear
  ```

> Tip: machine-friendly output
> `bijux config list --format json --no-pretty`

---

### `plugins` — Manage Plugins

Default install directory: `~/.bijux/.plugins` (override via `BIJUXCLI_PLUGINS_DIR`).

* **list** — `{"plugins":["...", ...]}`

  ```bash
  bijux plugins list
  ```

* **info \<name|path>**

  ```bash
  bijux plugins info my_plugin
  bijux plugins info ./path/to/my_plugin
  ```

* **install \<path>** (infers name from basename; use `--force` to overwrite)

  ```bash
  bijux plugins install ./path/to/my_plugin --force
  ```

* **check \<name|path>**

  ```bash
  bijux plugins check my_plugin
  bijux plugins check ./path/to/my_plugin
  ```

* **uninstall \<name>**

  ```bash
  bijux plugins uninstall my_plugin
  ```

* **scaffold \<name>** (outputs to current dir; `--force` to overwrite)

  ```bash
  mkdir -p ./temp_scaffold
  cd ./temp_scaffold
  bijux plugins scaffold my_plugin --template=../plugin_template --force
  cd ..
  bijux plugins install ./temp_scaffold/my_plugin --force
  ```

---

### `history` — Manage REPL History

* **list** (supports `--limit <n>`, `--group-by <field>`, `--filter <str>`, `--sort <field>`)
  Output shape: `{"entries":[ ... ]}`

  ```bash
  bijux history --limit 10
  ```

* **--export \<path>**

  ```bash
  bijux history --export ./history.json
  ```

* **--import \<path>**

  ```bash
  bijux history --import ./history.json
  ```

* **clear**

  ```bash
  bijux history clear
  ```

---

### `dev` — Developer Tools

* **di** — Dependency injection inventory
  Output shape: `{"factories":[...], "services":[...]}`

  ```bash
  bijux dev di
  ```

* **list-plugins**

  ```bash
  bijux dev list-plugins
  ```

---

## Built-in Commands

| Command   | Purpose                   | Example                      |
|-----------|---------------------------|------------------------------|
| `audit`   | Security/compliance audit | `bijux audit --dry-run`      |
| `docs`    | Generate API docs/specs   | `bijux docs --out spec.json` |
| `doctor`  | Environment health check  | `bijux doctor`               |
| `memory`  | In-memory key-value store | `bijux memory set key=val`   |
| `repl`    | Interactive shell         | `bijux repl`                 |
| `sleep`   | Pause execution           | `bijux sleep -s 5`           |
| `status`  | CLI status snapshot       | `bijux status`               |
| `version` | Display version info      | `bijux version`              |

Installed plugins appear as top-level commands (e.g., `my_plugin`).

---

## Configuration

**Default Paths** (overridable via env vars):

* Config: `~/.bijux/.env` (`BIJUXCLI_CONFIG`)
* History: `~/.bijux/.history` (`BIJUXCLI_HISTORY_FILE`)
* Plugins: `~/.bijux/.plugins` (`BIJUXCLI_PLUGINS_DIR`)

To customize (e.g., plugins dir), add to your shell profile:

```bash
export BIJUXCLI_PLUGINS_DIR=~/custom_plugins
```

**Resolution Precedence**

1) CLI flags → 2) Environment variables → 3) Config file → 4) Defaults

---

## End-to-End Examples

### Using Default Paths

```bash
# Clean up (optional)
bijux plugins uninstall my_plugin || true
rm -rf ./temp_scaffold ./usage_test_artifacts
mkdir -p ./usage_test_artifacts

# Scaffold and install
mkdir -p ./temp_scaffold
cd ./temp_scaffold
bijux plugins scaffold my_plugin --template=../plugin_template --force
cd ..
bijux plugins install ./temp_scaffold/my_plugin --force

# Verify
bijux plugins list
bijux plugins info my_plugin
bijux plugins check my_plugin

# Config
bijux config set core_timeout=30
bijux config get core_timeout
bijux config list

# Export config
bijux config export ./usage_test_artifacts/settings.env
bijux config export ./usage_test_artifacts/settings.json --format json

# History via REPL
bijux repl <<'EOF'
version
help
exit
EOF

# History ops
bijux history --limit 10
bijux history --export ./usage_test_artifacts/history.json
bijux history --import ./usage_test_artifacts/history.json

# Cleanup
bijux history clear
bijux config clear
bijux plugins uninstall my_plugin
rm -rf ./temp_scaffold

# Confirm
bijux plugins list
```

### Using Local Paths

```bash
# Clean up
rm -rf ./usage_test ./temp_scaffold
mkdir -p ./usage_test/plugins ./usage_test_artifacts

# Overrides
export BIJUXCLI_PLUGINS_DIR=./usage_test/plugins
export BIJUXCLI_CONFIG=./usage_test_artifacts/.env
export BIJUXCLI_HISTORY_FILE=./usage_test_artifacts/.history

# Scaffold and install
mkdir -p ./temp_scaffold
cd ./temp_scaffold
bijux plugins scaffold my_plugin --template=../plugin_template --force
cd ..
bijux plugins install ./temp_scaffold/my_plugin --force

# Verify
bijux plugins list
bijux plugins info my_plugin
bijux plugins check my_plugin

# Config
bijux config set core_timeout=30
bijux config get core_timeout
bijux config list

# Export config
bijux config export ./usage_test_artifacts/settings.env
bijux config export ./usage_test_artifacts/settings.json --format json

# History via REPL
bijux repl <<'EOF'
version
help
exit
EOF

# History ops
bijux history --limit 10
bijux history --export ./usage_test_artifacts/history.json
bijux history --import ./usage_test_artifacts/history.json

# Cleanup
bijux history clear
bijux config clear
bijux plugins uninstall my_plugin
rm -rf ./temp_scaffold

# Confirm
bijux plugins list

# Reset overrides (optional)
unset BIJUXCLI_PLUGINS_DIR
unset BIJUXCLI_CONFIG
unset BIJUXCLI_HISTORY_FILE
```

---

## Error Model (Structured)

When `--format` is set, errors are structured and emitted to **stderr** (unless `--quiet`):

```json
{
  "error": "message",
  "code": 2,
  "failure": "machine_readable_reason",
  "command": "subcommand path",
  "fmt": "json|yaml"
}
```

---

## Exit Codes

| Code | Meaning                |
|-----:|------------------------|
|  `0` | Success                |
|  `1` | General/internal error |
|  `2` | Usage/invalid argument |
|  `3` | Encoding/hygiene error |

Commands may extend with non-conflicting codes.

---

## Troubleshooting

* **Diagnostics:** `bijux doctor`
* **Verbosity:** add `--verbose` or `--debug`
* **Logs:** check stderr in `--debug` mode
* **Issues:** include `--debug` output when reporting bugs
