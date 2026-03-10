# Product Binaries

## Purpose
This document defines required binaries and routing ownership for product command execution.

## Scope
It covers only umbrella command routing and binary discovery requirements.

## Command Ownership
- `bijux` umbrella binary is owned by `bijux-cli`.
- Product runtime route: `bijux <tool> <...>` executes `bijux-<tool>`.
- Product control-plane route: `bijux dev <tool> <...>` executes `bijux-dev-<tool>`.
- Canonical known tools are declared in `docs/constitution/official_product_namespace_registry.json`.

## Required Binaries
| Product | Runtime binary | Control binary | Runtime package | Control package | Repository |
| --- | --- |
| agent | `bijux-agent` | `bijux-dev-agent` | `bijux-agent` | `bijux-dev-agent` | `bijux-agent` |
| atlas | `bijux-atlas` | `bijux-dev-atlas` | `bijux-atlas` | `bijux-dev-atlas` | `bijux-atlas` |
| dag | `bijux-dag` | `bijux-dev-dag` | `bijux-dag` | `bijux-dev-dag` | `bijux-dag` |
| dna | `bijux-dna` | `bijux-dev-dna` | `bijux-dna` | `bijux-dev-dna` | `bijux-dna` |
| gnss | `bijux-gnss` | `bijux-dev-gnss` | `bijux-gnss` | `bijux-dev-gnss` | `bijux-gnss` |
| rag | `bijux-rag` | `bijux-dev-rag` | `bijux-rag` | `bijux-dev-rag` | `bijux-rag` |
| rar | `bijux-rar` | `bijux-dev-rar` | `bijux-rar` | `bijux-dev-rar` | `bijux-rar` |
| vex | `bijux-vex` | `bijux-dev-vex` | `bijux-vex` | `bijux-dev-vex` | `bijux-vex` |

## Discovery Precedence
- Default precedence: configured product bin directories first, then `PATH`.
- Configured directories come from:
  - `BIJUXCLI_PRODUCT_BIN_DIR`
  - `BIJUXCLI_PRODUCT_BIN_DIRS`
- Precedence policy can be changed with `BIJUXCLI_PRODUCT_BIN_PRECEDENCE`.

## Failure Policy
- Missing required binaries return structured command errors.
- Allowlist policy for routed execution is controlled by `BIJUXCLI_ALLOWED_PRODUCT_BINS`.
- Optional strict major compatibility is controlled by `BIJUXCLI_ENFORCE_PRODUCT_MAJOR_MATCH=1`.
