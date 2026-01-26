# Precedence

## Purpose
This document defines the precedence rules that determine how configuration sources are resolved. It exists to remove ambiguity about which value wins when multiple sources provide the same setting.

## Scope
It covers CLI flags, environment variables, and configuration files. It does not describe the syntax of configuration files or the full set of available keys.

## Core Concepts
Precedence is a single, ordered chain. Values from later sources override earlier ones, and the final result is immutable for the duration of a command execution.

## How to Think About This
Think of precedence as a deterministic merge, not a negotiation. Every run computes the same result given the same inputs. Once computed, no later stage is allowed to override it.

## Invariants
1. CLI flags override environment variables and configuration files.
2. Environment variables override configuration files.
3. Defaults apply only when no other source provides a value.
4. Output format does not affect precedence.

## Failure Modes
If a source is invalid, resolution fails and the CLI returns a stable error. Resolution does not partially apply overrides; it either succeeds or fails entirely.

## Design Rationale
A single precedence chain prevents hidden overrides and makes behavior predictable in automation. It also keeps configuration debugging straightforward.

## Non-Goals
This document does not list every configuration key or describe file formats.

## References
- Implementation: `src/bijux_cli/core/precedence.py`
- Regression coverage: `tests/regression/test_precedence.py`
