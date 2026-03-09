# Adapter Interface Spec v0.1

## Purpose

Define the stable runtime adapter interface and invariants for built-in and external adapters.

## Adapter Identity

- `adapter_id` MUST be non-empty and stable.
- `adapter_version` MUST be non-empty and semantic-version-like.
- Adapter identity tuple is `adapter_id@adapter_version`.

## Required Interface Fields

- supported node kinds
- required effect set
- produced output schema version
- execution entrypoint that returns structured node results

## Lifecycle Expectations

1. descriptor validation before execution
2. execution with declared effects only
3. structured status and error normalization
4. deterministic metadata persistence for replay/verify surfaces

## Compatibility Rules

- duplicate adapter identity tuples are disallowed
- identity/version changes are treated as compatibility-relevant for replay/cache
- adapter metadata does not alter canonical graph identity

## Conformance

Adapter implementations must pass runtime adapter conformance and registry capability contracts.
