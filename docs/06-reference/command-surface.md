# Command Surface

## Purpose

This page lists the current command surface, key subcommand groups, REPL session
controls, and stable exit codes.

```mermaid
flowchart TD
    A[bijux] --> B[Top-level command]
    B --> C[Subcommand group]
    C --> D[Flags and arguments]
    D --> E[Exit code and output]
```

```mermaid
flowchart LR
    A[CLI invocation] --> B[Top-level commands]
    A --> C[REPL controls]
    A --> D[Exit codes]
```

## Built-In Top-Level Commands

| Command | Purpose |
| --- | --- |
| `cli` | Canonical runtime namespace for explicit subcommands such as `paths` and `self-test` |
| `audit` | Diagnostics audit report |
| `config` | Configuration management |
| `docs` | Documentation tools |
| `doctor` | Environment diagnostics |
| `help` | Global help |
| `history` | REPL history tools |
| `memory` | In-memory key/value tools |
| `plugins` | Plugin management |
| `repl` | Interactive shell |
| `completion` | Shell completion output |
| `sleep` | Sleep for a duration |
| `status` | CLI status probe |
| `version` | CLI version |

## Common Subcommand Groups

### `cli`

`status`, `paths`, `config`, `self-test`, `plugins`

### `config`

`list`, `get`, `set`, `unset`, `export`, `load`, `reload`, `clear`

### `plugins`

`list`, `info`, `inspect`, `check`, `enable`, `disable`, `install`,
`uninstall`, `scaffold`, `doctor`, `reserved-names`, `where`, `explain`,
`schema`

### `history`

`clear`

### `memory`

`clear`, `delete`, `get`, `list`, `set`

## Routed Namespaces

Some valid command families are routed dynamically and are not part of the
static built-in help inventory:

- `bijux dev <product> ...` for maintainer control planes such as
  `bijux dev cli ...`
- `bijux <product> ...` for adjacent Bijux product runtimes when the matching
  binary is available and allowed by the current routing policy

Use [Integrations And Routed Runtimes](integrations-and-routed-runtimes.md) for
the routed-product and maintainer-route rules.

## REPL Session Controls

When using `bijux repl`, the documented session controls are:

- `:help <command>`
- `:set trace on|off`
- `:set quiet on|off`
- `:set format json|yaml|text`
- `:plugin reload`
- `:exit`

## Stable Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Internal or general runtime failure |
| `2` | Usage or user-input error |
| `3` | Serialization, ASCII, or encoding error |
| `130` | User abort or interruption |

## Honest Limit

This page is a command inventory, not a full behavior tutorial. For usage
workflows, go to [User Guide](../03-user-guide/index.md).
