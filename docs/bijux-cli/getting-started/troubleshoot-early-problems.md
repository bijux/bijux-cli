# Troubleshoot Early Problems

## Purpose

Most early failures are not deep runtime bugs. They are usually install
conflicts, path ambiguity, unsupported assumptions, or trying to use a feature
before verifying the basics.

```mermaid
flowchart TD
    A[Early failure] --> B{Does bijux resolve?}
    B -->|No| C[Fix installation or PATH]
    B -->|Yes| D{Does doctor report conflicts?}
    D -->|Yes| E[Remove duplicate installs or pin the runtime]
    D -->|No| F{Is the expectation supported?}
    F -->|No| G[Adjust workflow]
    F -->|Yes| H[Move to a guide or issue investigation]
```

This flowchart gives the intended troubleshooting order. It starts with binary
resolution, then moves through install health, and only after that asks whether
the underlying expectation is actually supported.

```mermaid
mindmap
  root((Early problems))
    Install conflicts
      cargo and pip both present
      stale wrappers
    Usage mistakes
      scraping text output
      assuming plugins are sandboxed
    Environment limits
      unsupported platform
      wrong binary in PATH
```

The mindmap groups the common causes behind that flow: install identity
problems, usage mistakes, and environment limits. Reading failures this way
helps keep early debugging concrete instead of speculative.

## First Checks

Run:

```bash
bijux version
bijux cli paths
bijux doctor
```

These commands are still the fastest way to separate environment problems from
command-level problems.

## Common Real Causes

- more than one `bijux` install exists and the wrong one wins in `PATH`
- a wrapper script or shell cache still points at an older binary
- automation expects text output to stay parser-friendly
- the workflow assumes plugin isolation that Bijux does not provide
- the environment is outside the supported platform envelope, including current
  Windows hosts

## When To Go Deeper

Move to the fuller docs when the basics are already proven:

- [Installation and recovery](../getting-started/installation-and-recovery.md)
- [Command surface](../reference/command-surface.md)
- [Configuration and output](../user-guide/configuration-and-output.md)
- [Plugins and extensions](../user-guide/plugins-and-extensions.md)
- [Testing and evidence](../../bijux-dev/development/testing-and-evidence.md)

## Closing Rule

Fix install identity first, then command behavior, then higher-level workflows.
Skipping that order usually wastes time.
