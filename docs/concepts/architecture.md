# Architecture

## Components

- CLI entry: parses args into an intent
- Policy resolution: computes effective flags and output rules
- Runtime: DI, plugins, and command dispatch
- Emission: writes payloads via resolved routing
- Services: config, history, diagnostics, plugin registry

The execution path is linear: Intent -> Policy -> Runtime -> Exit.

## Dependency direction

- Core depends on nothing
- Infra depends on core only
- Services depend on core and infra
- CLI depends on services only
- App wires everything

## Contract ownership

- Core: cross-cutting behavior and infra-facing interfaces
- Services: service protocols and expectations
- Infra: concrete adapters for core interfaces

## Plugin pipeline

Ordered stages:

1. Discover
2. Validate metadata
3. Register
4. Activate (lazy)
5. Unload if applicable

This order is enforced in code and reviews.
