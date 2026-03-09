# What Is Bijux Dag

## Purpose
Explain what bijux-dag is, what problem it solves, and why its deterministic model matters.

## Context
This is the first technical entrypoint for new readers. It defines the system identity and its problem space before command details or architecture internals.

## Explanation
Bijux-dag is a deterministic workflow runtime and contract-driven toolchain for directed acyclic graph (DAG) computation.

The practical shorthand "Git for computation graphs" means:
- graph definitions are explicit, inspectable, and comparable
- run outcomes can be traced to defined inputs and graph structure
- replay and diff are first-class system operations, not afterthought scripts

Modern pipeline operations often degrade because they are opaque and drift-prone:
- hidden mutable state changes behavior over time
- retries and reruns are hard to reason about
- output differences are visible, but causes are unclear
- operations teams cannot confidently answer "what changed and why"

Bijux-dag addresses this with deterministic pipeline control:
- explicit graph and execution semantics
- identity-backed run and artifact modeling
- replay and diff as core workflow primitives
- inspect surfaces for debugging and verification

Determinism is central because it converts workflow operation from guesswork to controlled analysis.
Deterministic behavior improves:
- repeatability: the same inputs and graph produce stable behavior
- debuggability: divergences are attributable, not mysterious
- trust: guarantees can be documented, tested, and verified

## Examples
```bash
# Run a DAG
bijux-dag run --dag ./examples/basic.dag.json

# Replay the same DAG/run context
bijux-dag replay --run-id RUN_20260309_001

# Compare two runs
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002
```

## Guarantees
- Bijux-dag treats replay, diff, and inspect as core product behavior.
- Deterministic control is a system objective, not optional guidance.
- The product framing in this document is aligned to runtime and specification sections.

## Limitations
- "Git for computation graphs" is a conceptual analogy, not feature parity with Git.
- Determinism does not imply universal backend or environment equivalence.
- This document does not define low-level contracts; those are covered in specification docs.

## Related
- `docs/01-introduction/02-mission.md`
- `docs/01-introduction/03-design-principles.md`
- `docs/01-introduction/04-core-concepts.md`
- `docs/06-specification/01-dag-model.md`
