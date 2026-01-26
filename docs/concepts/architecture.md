# Architecture

## Purpose
This document explains the major components of bijux-cli and how they fit together. It exists to give you a practical map of responsibilities so you know where behavior lives and where it must not live.

## Scope
The architecture description focuses on CLI execution and related services such as configuration, plugins, and output routing. It does not describe internal code structures in detail or provide API documentation.

## Core Concepts
Bijux-cli is organized as a pipeline: argument parsing builds an intent, policy resolution builds an immutable runtime configuration, runtime services are initialized, commands execute, and output is emitted exactly once. This structure prevents late overrides and keeps behavior deterministic.

## How to Think About This
Treat the architecture as a boundary of responsibilities rather than a code layout. Parsing is data construction, policy resolution is the single point of truth, runtime initialization is a controlled side-effect, and emission is the final output step. If you are unsure where logic belongs, place it in the earliest stage that can safely own it.

## Invariants
The architecture enforces one-way flow: later stages must not mutate earlier decisions. Output routing is decided in core, not in infra, and command handlers must not access raw configuration or environment state directly.

## Failure Modes
Architectural violations show up as nondeterministic behavior, inconsistent output formats, or broken CLI/API parity. The system treats these as defects, not as acceptable edge cases, and tests explicitly guard against them.

## Design Rationale
A linear architecture is easier to test, reason about, and evolve. It also makes it clear how guarantees are enforced, which is critical for both users and plugin authors.

## Non-Goals
This document does not enumerate every module or class. It only describes the architectural responsibilities that define the CLI's contract.

## References
- [Execution model](execution-model.md)
- [Decision rules](../architecture/decision-rules.md)
