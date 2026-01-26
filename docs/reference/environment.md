# Environment

## Purpose
This document guarantees environment variable names and meanings.

## Scope
It lists supported variables only.

## Core Concepts
- Environment overrides config files.

## Invariants
- Environment names are stable.

## Execution
| Variable | Purpose |
| --- | --- |
| BIJUXCLI_FORMAT | Output format override |
| BIJUXCLI_LOG_LEVEL | Log level override |
| BIJUXCLI_COLOR | Color mode override |
| BIJUXCLI_CONFIG | Config file path override |
| BIJUXCLI_PLUGINS_DIR | Plugin directory override |

## Failure Modes
- Invalid values exit with code 2 or 3 depending on error class.

## Design Rationale
- Alternatives: implicit environment variables.
- Rejected because they are hard to audit.

## Non-Goals
- Hidden or undocumented variables.
