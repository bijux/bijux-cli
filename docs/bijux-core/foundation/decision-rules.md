---
title: Decision Rules
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Decision Rules

When a reader is unsure where to start, the repository should offer routing
rules instead of making them guess.

```mermaid
flowchart TD
    A[New question or change] --> B{Is it about runtime command behavior?}
    B -->|Yes| C[Route to CLI handbook]
    B -->|No| D{Is it about graph execution, replay, or artifacts?}
    D -->|Yes| E[Route to DAG handbook]
    D -->|No| F{Is it about repository gates, release, or docs publishing?}
    F -->|Yes| G[Route to Maintainer handbook]
    F -->|No| H[Stay in repository handbook until ownership conflict is resolved]
```

## Routing Rules

- if the question is about runtime command behavior, go to [CLI](../../bijux-cli/index.md)
- if the question is about graph execution, replay, or artifacts, go to
  [DAG](../../bijux-dag/index.md)
- if the question is about repository gates, docs publishing, or release
  control, go to [Maintainer](../../bijux-dev/index.md)
- if two branches both seem to own the answer, stay in the repository handbook
  until the ownership conflict is resolved

## Escalate To The Repository Handbook When

- the change affects more than one product handbook
- the question involves root files or shared automation
- the docs tree itself is inconsistent

## Next Reads

- [Package Map](package-map.md)
- [Repository Handbook](../index.md)
- [Maintainer Handbook](../../bijux-dev/index.md)
