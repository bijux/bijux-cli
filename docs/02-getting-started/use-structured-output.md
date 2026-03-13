# Use Structured Output

## Goal

Move from human inspection to scriptable behavior. Bijux is most useful when you
request explicit output formats instead of scraping styled terminal text.

```mermaid
flowchart TD
    A[Command result] --> B{Need automation?}
    B -->|Yes| C[Use --format json or --format yaml]
    B -->|No| D[Use default human-readable output]
    C --> E[Parse a stable envelope]
```

```mermaid
sequenceDiagram
    participant U as User
    participant C as CLI
    participant J as JSON consumer
    U->>C: bijux status --format json --no-pretty
    C-->>U: compact JSON
    U->>J: pass JSON to script
    J-->>U: stable machine-readable handling
```

## First Structured Command

Run:

```bash
bijux status --format json --no-pretty
```

If you prefer YAML for manual inspection:

```bash
bijux status --format yaml
```

## Working Rule

For automation:

- prefer `json`
- add `--no-pretty` when compact output matters
- rely on exit codes and structured output together

For interactive work:

- human-readable text is fine
- YAML can be useful when reading nested structures manually

## Honest Limit

Structured output improves reliability, but it does not remove the need to
check exit codes. A script that ignores command failure and only parses output
is still brittle.

## Read Next

Continue to [Troubleshoot Early Problems](troubleshoot-early-problems.md).
