# Product Binaries

## Purpose
This document defines required binaries and routing ownership for product command execution.

## Scope
It covers only umbrella command routing and binary discovery requirements.

## Command Ownership
- `bijux` umbrella binary is owned by `bijux-cli`.
- Product runtime route: `bijux atlas <...>` executes `bijux-atlas`.
- Product control-plane route: `bijux dev atlas <...>` executes `bijux-dev-atlas`.

## Required Binaries
| Product | Required binaries |
| --- | --- |
| atlas | `bijux-atlas`, `bijux-dev-atlas` |

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
