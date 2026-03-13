# Configuration And State

`bijux-cli` is stateful, but the architecture tries to make that state explicit and inspectable.

## Scope

This page covers:

- configuration precedence
- runtime path resolution
- the small set of durable state files the runtime manages

## Resolution Model

```mermaid
flowchart TD
    A[Defaults] --> B[Compatibility config file]
    B --> C[Environment variables]
    C --> D[Explicit CLI flags]
    D --> E[Resolved state paths]
```

```mermaid
graph TD
    Home[Resolved home directory] --> Config[config file]
    Home --> History[history file]
    Home --> Memory[memory file]
    Home --> Plugins[plugins directory]
    Plugins --> Registry[plugin registry file]
```

## Current State Files

The runtime currently works with a small number of state locations:

- configuration file
- history file
- memory file
- plugin directory
- plugin registry file

The exact path rules matter because the runtime treats path resolution as part of observable behavior, not as an invisible implementation detail.

## Why This Is Architectural

State handling is not just storage. It affects:

- reproducibility
- diagnostics
- plugin health checks
- compatibility behavior
- the difference between a safe reset and silent data loss

## Configuration Philosophy

The architecture prefers:

- explicit precedence
- text formats that can be inspected and repaired
- narrow data models over arbitrary blobs
- diagnostic reporting when state is malformed

It does not promise:

- every historical compatibility key forever
- silent repair of all malformed user state

## Runtime State Flow

```mermaid
sequenceDiagram
    participant U as User input
    participant P as Path resolver
    participant S as State readers
    participant R as Runtime

    U->>P: env and flags
    P->>S: resolved file paths
    S->>R: parsed state or errors
    R-->>U: command output plus diagnostics
```

```mermaid
stateDiagram-v2
    [*] --> Resolved
    Resolved --> Loaded
    Loaded --> Healthy
    Loaded --> Degraded
    Degraded --> Repaired
    Repaired --> Healthy
```

## Honest Constraint

The state model is intentionally file-oriented and local-first.

That keeps the runtime understandable, but it also means filesystem failures, permissions, and malformed local state are first-class architecture concerns rather than edge cases.
