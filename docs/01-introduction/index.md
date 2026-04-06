# Introduction

## Purpose

This section is the shortest honest path to understanding what `bijux-cli` is,
what it promises, and where it currently stops.

```mermaid
flowchart TD
    A[What is Bijux?] --> B[What happens on the first run?]
    B --> C[How should I think about command execution?]
    C --> D[Where does the project fit well?]
    D --> E[What limits should I know before I commit to it?]
```

This flowchart is the intended reading order for the section. It starts with
identity, then moves through first-use behavior and execution shape before
ending on the practical limits that decide whether Bijux fits your workflow.

```mermaid
mindmap
  root((Introduction))
    Identity
      Rust runtime ownership
      Python distribution surface
    Operation
      Deterministic flags
      Shared CLI and REPL law
    Fit
      Scriptable tools
      Plugin-driven command sets
    Limits
      Windows-aware internals, unsupported host
      No plugin sandbox
```

The mindmap is the short summary of what this section establishes: who owns the
runtime today, how command execution behaves, where the project fits well, and
which limits you should carry forward into the rest of the docs. The Windows
note is deliberately precise: some internals are Windows-aware now, but the
supported host contract still excludes Windows.

## Read This Set In Order

1. [What Bijux Is](what-bijux-is.md)
2. [First Run](first-run.md)
3. [Command Model](command-model.md)
4. [Limits And Guarantees](limits-and-guarantees.md)

## Writing Standard

These pages are intentionally narrow:

- they describe the current system, not an aspirational roadmap
- they prefer clear limits over broad claims
- they link into deeper guides and architecture only after the basics are clear

## Next Step

If you already know you want installation details, go to
[First Run](first-run.md). If you need the deeper system map, continue to
[System Overview](../04-architecture/system-overview.md).
