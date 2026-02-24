# Commands

## Purpose
This document guarantees the canonical command list.

## Scope
It lists commands and subcommands only.

## Core Concepts
- Each command appears once.

## Invariants
- The list is exhaustive.

## Execution
### Top-level commands

| Command | Purpose |
| --- | --- |
| atlas | Bijux Atlas runtime command proxy |
| audit | Diagnostics audit report |
| config | Configuration management |
| dev | Developer tools |
| docs | Documentation tools |
| doctor | Environment diagnostics |
| help | Global help |
| history | REPL history tools |
| memory | In-memory key/value tools |
| plugins | Plugin management |
| repl | Interactive shell |
| sleep | Sleep for a duration |
| status | CLI status probe |
| version | CLI version |

### Config subcommands

| Subcommand | Purpose |
| --- | --- |
| list | List all config keys |
| get | Get a single key |
| set | Set a key/value pair |
| unset | Remove a key |
| export | Export config to a file |
| load | Load config from a file |
| reload | Reload config from disk |
| clear | Remove all config keys |

### Plugins subcommands

| Subcommand | Purpose |
| --- | --- |
| list | List installed plugins |
| info | Show plugin metadata |
| check | Validate plugin metadata |
| install | Install a plugin directory |
| uninstall | Remove a plugin |
| scaffold | Generate a plugin from a template |

### History subcommands

| Subcommand | Purpose |
| --- | --- |
| clear | Clear history entries |
| service | History service info |

### Memory subcommands

| Subcommand | Purpose |
| --- | --- |
| clear | Clear memory entries |
| delete | Delete a key |
| get | Get a key |
| list | List keys |
| set | Set a key/value pair |
| service | Memory service info |

### Dev subcommands

| Subcommand | Purpose |
| --- | --- |
| atlas | Bijux Atlas control-plane proxy |
| di | DI graph summary |
| list-plugins | List plugin discovery results |
| service | Dev service info |

## Failure Modes
- Missing entries indicate documentation defects.

## Design Rationale
- Alternatives: command lists embedded in guides.
- Rejected because they go stale.

## Non-Goals
- Full command usage text.
