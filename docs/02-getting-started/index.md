# Getting Started

## Purpose

This section is the practical onboarding path after the introduction pages. It
shows how to get a working installation, verify it, run a few real commands,
and avoid the most common early misunderstandings.

```mermaid
flowchart TD
    A[Install the runtime] --> B[Verify the active binary]
    B --> C[Run core commands]
    C --> D[Request structured output]
    D --> E[Handle early failures]
```

```mermaid
mindmap
  root((Getting Started))
    Install
      Cargo
      pip
      pipx
    Verify
      version
      paths
      doctor
    Use
      help
      status
      structured output
    Recover
      duplicate installs
      path confusion
      unsupported expectations
```

## Read This Set In Order

1. [Install And Verify](install-and-verify.md)
2. [Run Your First Commands](run-your-first-commands.md)
3. [Use Structured Output](use-structured-output.md)
4. [Troubleshoot Early Problems](troubleshoot-early-problems.md)

## Scope

These pages are intentionally narrow:

- they cover the first successful operational path
- they do not replace the full installation or command references
- they avoid feature tours that belong in guides or architecture

## Next Step

If you have not read the project identity and limits yet, start with
[Introduction](../01-introduction/index.md). If you already know what Bijux is,
continue to [Install And Verify](install-and-verify.md).
