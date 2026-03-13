# Interactive Shell And History

## Goal

Use the REPL when interactive exploration is faster than retyping commands in a
new shell process every time.

```mermaid
flowchart TD
    A[CLI mode] --> B{Need repeated exploration?}
    B -->|No| C[Stay in normal CLI]
    B -->|Yes| D[Open repl]
    D --> E[Run commands]
    E --> F[Use history and session controls]
```

```mermaid
sequenceDiagram
    participant U as User
    participant R as REPL
    U->>R: bijux repl
    U->>R: status
    R-->>U: command output
    U->>R: :set format json
    U->>R: doctor
    R-->>U: JSON output
```

## Start The REPL

```bash
bijux repl
```

Useful session controls:

- `:help <command>`
- `:set format json|yaml|text`
- `:set quiet on|off`
- `:set trace on|off`
- `:exit`

## History And Repetition

The REPL is useful when:

- you are exploring several related commands
- you want consistent session-local formatting
- you want command history without rebuilding shell pipelines repeatedly

## Honest Limit

The REPL follows the same command law as the CLI, but it is still an
interactive shell. For automation, use normal CLI invocations with explicit
output formats.

## Where To Go Deeper

- [Command surface](../06-reference/command-surface.md)
- [Integrations and routed runtimes](../06-reference/integrations-and-routed-runtimes.md)
- [Routing and surfaces architecture](../04-architecture/routing-and-surfaces.md)
