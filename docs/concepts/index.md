# Concepts

## Purpose
This section defines the guarantees and mental models that make bijux-cli predictable. It exists to explain the rules the CLI must follow so you can reason about behavior without reading source code.

## Scope
Concepts documents the execution model, precedence rules, exit behavior, logging semantics, and plugin lifecycle. It does not provide step-by-step usage instructions or command reference tables.

## Audience
Engineers who want to understand why the CLI behaves the way it does should start here. This section is designed to eliminate ambiguity and to provide stable guarantees that tests enforce.

## What You Will Find
Each concept page states its purpose, scope, invariants, and failure modes. The goal is to clarify where decisions are made and what is explicitly forbidden.

## Index
- [Architecture](architecture.md)
- [Execution model](execution-model.md)
- [Precedence](precedence.md)
- [Exit policy](exit-policy.md)
- [Logging](logging.md)
- [Plugin lifecycle](plugin-lifecycle.md)
