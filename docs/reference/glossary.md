# Glossary

## Purpose
This document guarantees definitions of official terms.

## Scope
It defines terms used across docs and code.

## Core Concepts
- Terms are authoritative and stable.

## Invariants
- Each term has a single definition.

## Execution
- Intent: fully resolved command with flags and context.
- Policy: output and logging rules derived from precedence.
- ExitIntent: structured exit decision with code and payload.
- Registry: plugin registry storing metadata and states.
- Precedence: ordering rules for configuration resolution.

## Failure Modes
- Ambiguous terms must be rewritten, not duplicated.

## Design Rationale
- Alternatives: inline definitions in every doc.
- Rejected because they drift.

## Non-Goals
- Glossary of external ecosystem terms.
