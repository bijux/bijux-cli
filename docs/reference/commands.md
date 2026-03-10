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
| agent | Bijux Agent runtime command proxy |
| atlas | Bijux Atlas runtime command proxy |
| audit | Diagnostics audit report |
| config | Configuration management |
| dag | Bijux DAG runtime command proxy |
| dev | Developer tools |
| dna | Bijux DNA runtime command proxy |
| docs | Documentation tools |
| doctor | Environment diagnostics |
| gnss | Bijux GNSS runtime command proxy |
| help | Global help |
| history | REPL history tools |
| memory | In-memory key/value tools |
| plugins | Plugin management |
| rag | Bijux RAG runtime command proxy |
| rar | Bijux RAR runtime command proxy |
| repl | Interactive shell |
| sleep | Sleep for a duration |
| status | CLI status probe |
| version | CLI version |
| vex | Bijux VEX runtime command proxy |

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
| agent | Bijux Agent control-plane command proxy |
| atlas | Bijux Atlas control-plane proxy |
| dag | Bijux DAG control-plane command proxy |
| di | DI graph summary |
| dna | Bijux DNA control-plane command proxy |
| gnss | Bijux GNSS control-plane command proxy |
| list-products | List required product binaries and resolved paths |
| list-plugins | List plugin discovery results |
| rag | Bijux RAG control-plane command proxy |
| rar | Bijux RAR control-plane command proxy |
| service | Dev service info |
| vex | Bijux VEX control-plane command proxy |

## Failure Modes
- Missing entries indicate documentation defects.

## Design Rationale
- Alternatives: command lists embedded in guides.
- Rejected because they go stale.

## Non-Goals
- Full command usage text.
