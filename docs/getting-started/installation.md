# Installation

## Purpose
This document guarantees how to install bijux-cli.

## Scope
It covers pip and developer installs only.

## Core Concepts
- Install via pip for users.
- Install editable for contributors.

## Invariants
- Python 3.11 or newer is required.

## Execution
```bash
pip install bijux-cli
```

Developer install:

```bash
hatch shell
pip install -e .
```

Verify:

```bash
bijux --help
bijux version
```

## Failure Modes
- Missing Python 3.11+ prevents install.
- Missing PATH entry prevents `bijux` execution.

## Design Rationale
- Alternatives: system package managers.
- Rejected because release cadence is Python-first.

## Non-Goals
- OS package manager instructions.
