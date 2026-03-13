# Routing And Surfaces

The runtime exposes more than one user-facing surface, but they share a common routing model.

## Surfaces

```mermaid
graph TD
    A[Root CLI]
    B[Canonical cli namespace]
    C[REPL]
    D[Python package facade]
    E[Maintainer and product routes]

    A --> R[Shared route model]
    B --> R
    C --> R
    D --> R
    E --> R
```

```mermaid
flowchart LR
    RootAlias[Root aliases like plugins inspect] --> Canonical[cli plugins inspect]
    Canonical --> Handler[command handler]
    Handler --> Output[result]
```

## Current Surfaces

### Root CLI

This is the shortest built-in user-facing surface, for example:

- `bijux status`
- `bijux config list`
- `bijux plugins list`
- `bijux doctor`

### Canonical `cli` Namespace

This is the explicit runtime namespace for routes that either do not exist at
the root or are useful to inspect in their normalized form:

- `bijux cli paths`
- `bijux cli plugins inspect`
- `bijux cli self-test`

The canonical namespace matters because it exposes the normalized route shape
even when shorter root commands also exist.

### Routed Maintainer And Product Surfaces

These are valid routed surfaces, but they are not part of the static built-in
command inventory:

- `bijux dev cli ...` for the maintainer control-plane
- `bijux <product> ...` for adjacent Bijux products when the matching runtime
  binary is available and allowed

### REPL

The REPL is an interface, not a separate architecture.

It uses the same routing and execution model as the CLI as far as practical.

### Python Package Facade

The Python package is a distribution surface that calls into the Rust runtime or its bridge layer. It is not the authoritative source of routing rules.

## Alias Policy

```mermaid
flowchart TD
    A[User input] --> B{Known alias or routed namespace?}
    B -->|yes| C[Normalize route]
    B -->|no| D[Keep parsed route]
    C --> E[Dispatch]
    D --> E
```

```mermaid
sequenceDiagram
    participant U as User
    participant P as Parser
    participant N as Normalizer
    participant H as Handler

    U->>P: plugins inspect sample
    P->>N: parsed path
    N->>H: cli/plugins/inspect
    H-->>U: plugin inspection payload
```

## Help Surface

Help is part of the routed surface, not an afterthought.

The project treats help output as a contract because:

- users rely on it as the first explanation of the command surface
- snapshot tests catch accidental drift
- bad examples in help become real usability defects

## Honest Boundary

The route model is shared, but not every route is equal:

- some routes are public runtime behavior
- some routes are normalized compatibility aliases
- some routes are routed product or maintainer namespaces

The architecture works because those categories are kept separate in code and documentation.
