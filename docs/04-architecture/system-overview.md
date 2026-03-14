# System Overview

`bijux-cli` is a Rust-owned command runtime with Python packaging and maintainer-facing support crates around it.

The project is no longer best understood as "a Python CLI being ported." The current system is already centered on the Rust runtime. Python remains important, but mainly as a packaging surface and compatibility surface.

## Core Shape

```mermaid
flowchart LR
    User[User or automation] --> CLI[bijux binary]
    User --> PY[Python package entrypoint]
    CLI --> Runtime[bijux-cli runtime]
    PY --> Runtime
    Runtime --> State[Config and state files]
    Runtime --> Plugins[Plugin registry and local plugin files]
    Maintainer[Maintainer] --> DEV[bijux-dev-cli]
    DEV --> Runtime
    DEV --> Repo[Repository diagnostics]
```

This diagram shows the main system shape around the runtime. Users, the Python
distribution, and maintainers all approach the system differently, but the Rust
runtime remains the center that ties those paths together.

```mermaid
graph TD
    A[Input] --> B[Routing]
    B --> C[Policy]
    C --> D[Execution]
    D --> E[Output envelope]
    E --> F[Stdout or stderr]
    F --> G[Exit code]
```

The second diagram compresses runtime behavior into its essential command path.
It is the short explanation for why routing, policy, execution, and emission
are treated as one coherent model throughout the docs.

## What The Runtime Owns

The runtime owns:

- command parsing and route normalization
- policy resolution for global flags
- execution semantics
- output formatting
- exit behavior
- configuration and state-path resolution
- plugin installation, inspection, and health checks
- REPL behavior

## What The Runtime Does Not Claim

The runtime does not currently claim:

- sandboxed plugin execution
- an in-process stable plugin ABI
- Windows support
- full removal of Python-facing compatibility surfaces
- that every maintainer-facing report is part of the public user contract

The Windows item is a support-boundary statement, not a denial that the code
has evolved. The runtime already contains Windows-aware binary discovery,
state-path resolution, and some filesystem handling, but the end-to-end product
contract is still blocked by POSIX-oriented plugin scaffolding and shell
completion integration.

## Design Priorities

The architecture is optimized for:

- determinism over convenience magic
- explicit state over hidden side effects
- executable contracts over prose-only claims
- Rust runtime ownership with thin packaging wrappers
- stable command behavior even while internal implementation changes

## Non-Priorities

The system intentionally does not optimize for:

- plugin isolation
- dynamic runtime mutation without validation
- broad platform abstraction at any cost
- preserving every historical implementation detail from earlier Python ownership

## Public And Internal Surfaces

```mermaid
flowchart TD
    A[Public user contract]
    B[Compatibility surface]
    C[Maintainer control-plane]
    D[Internal implementation]

    A --> A1[CLI commands]
    A --> A2[Output formats]
    A --> A3[Exit codes]
    A --> A4[Plugin manifest contract]

    B --> B1[Python package facade]
    B --> B2[Stable release overlap checks]

    C --> C1[bijux-dev-cli]
    C --> C2[repo and release diagnostics]

    D --> D1[module layout]
    D --> D2[crate boundaries]
    D --> D3[test harnesses]
```

This diagram separates what the repository treats as public from what remains
internal. That boundary is important because the project promises command and
contract behavior more strongly than it promises internal crate layout.

```mermaid
sequenceDiagram
    participant U as User
    participant C as CLI surface
    participant R as Runtime
    participant O as Output layer

    U->>C: invoke command
    C->>R: normalized intent
    R->>R: resolve policy and execute
    R->>O: structured result or error
    O-->>U: stdout/stderr plus exit code
```

The sequence diagram shows the user-visible runtime path from invocation to
output. It reinforces that the caller experiences one consistent execution
chain, even though several internal layers participate.

## Honest Status

The architecture is coherent, but it is not minimal by accident. Some complexity is deliberate:

- the repository still carries multiple distribution surfaces
- the plugin system has real lifecycle and filesystem concerns
- the maintainer control-plane is intentionally separate from the end-user runtime

That complexity should be documented directly, not hidden behind slogans about elegance.
