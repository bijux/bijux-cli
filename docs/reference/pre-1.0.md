# What can change before 1.0

## Purpose
We state what may change before 1.0 so you can plan integrations safely.

## Scope
This list applies only to pre-1.0 releases. After 1.0, we treat these as stable contracts.

## Changes we may introduce
1. Plugin metadata schema fields and validation rules.
2. REPL UX details (completions, prompts, and formatting).
3. Config extensions and new config keys.

## Guarantees even before 1.0
- Exit codes stay stable for documented errors.
- Precedence rules stay stable once documented.
- CLI/API parity remains enforced for critical flows.

## Failure Modes
If we must break a pre-1.0 surface, we will:
- document it in the changelog,
- update the relevant concept doc,
- provide a short migration note.

## Non-Goals
- We do not guarantee backward compatibility for undocumented behavior.
