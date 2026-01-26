# Architecture

## Purpose
This section tells you where decisions are allowed and where they are forbidden.

## Scope
It covers decision boundaries and policy ownership only.

## What problem this solves
Without written rules, policy moves into random files and tests become brittle.

## Why you should care
If you extend bijux-cli, you must know which file owns each decision.

## What confusion this removes
It removes ambiguity about where policy and exit behavior live.

## Guarantees
Bijux guarantees:
1. Policy is resolved only in core.
2. Exit behavior is resolved only in core.

## How to Think About This
Treat these rules as enforcement, not guidance.

## Common Misunderstandings
- "Rules are suggestions." They are not.

## Execution
- Decision rules: decision-rules.md

## Failure Modes
- Boundary violations block review or break tests.

## Design Rationale
We deliberately chose strict boundaries to prevent drift.
Why not allow exceptions? Exceptions become the new normal.

## Non-Goals
- Feature documentation.
