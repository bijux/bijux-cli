# Contributor Mental Model

## Purpose
This document explains the mental model contributors should use when changing bijux-cli. It exists to keep changes aligned with the core execution model and to prevent logic from leaking into the wrong layer.

## Scope
It applies to contributors working on CLI commands, runtime flow, and plugin behavior. It does not describe individual commands or configuration keys.

## Core Concepts
The CLI is a pipeline: intent, policy, runtime, execution, and emission. Each stage owns specific responsibilities, and crossing those boundaries creates bugs that are difficult to test or detect.

## How to Think About This
When you add or change behavior, start by choosing the correct stage. If a decision belongs to policy, do not implement it in a command handler. If output routing is involved, keep it in core, not infra.

## Invariants
Policy must be resolved exactly once. Commands must not read raw environment variables directly. Output routing must remain a single decision point in core.

## Failure Modes
Violating the pipeline leads to nondeterministic output, inconsistent exit codes, and broken CLI/API parity. These issues are treated as regressions, even if they appear to work locally.

## Design Rationale
The mental model is intentionally strict because it enables strong guarantees and reliable tests. It trades short-term flexibility for long-term correctness.

## Non-Goals
This document does not define coding style rules or release procedures.

## References
- [Concepts overview](../concepts/index.md)
- [Architecture walk-through](../architecture/walkthrough.md)
