# Global Flags

## Purpose
Define stable global flags, parsing placement, and precedence semantics.

## Scope
This document covers root-level flags that apply across command namespaces.

## Core Concepts
- Global flags are public API.
- Flag semantics are independent of command implementation.

## Invariants
- Supported global flags are:
  - `--help`
  - `--version`
  - `--format`
  - `--pretty` and `--no-pretty`
  - `--quiet`
  - `--log-level`
  - `--color` and `--no-color`
- Global flags may appear before or after namespace segments when syntactically unambiguous.
- Final effective policy precedence is: `flags -> env -> config -> defaults`.
- `--help` and `--version` are short-circuit flags.

## Failure Modes
- Ambiguous or invalid flag values return a usage error.
- Unknown global flags return a usage error.

## Design Rationale
- Stable flag semantics support long-lived scripts and wrappers.

## Non-Goals
- Command-specific option definitions.
