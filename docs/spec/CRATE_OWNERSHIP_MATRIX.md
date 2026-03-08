# Crate Ownership Matrix

## Runtime

- ownership: runtime execution kernel and adapter/runtime boundaries
- not owned: CLI routing, release evidence authority, schema source-of-truth ownership

## Core

- ownership: graph model, canonicalization, validation semantics
- not owned: runtime execution backend behavior or adapter implementations

## Artifacts

- ownership: artifact identity, lineage, storage/transport primitives
- not owned: command routing and CLI argument parsing

## App

- ownership: command routing, operator UX surfaces, output rendering
- not owned: low-level artifact storage primitives

## Dev Governance

- ownership: release evidence, policy checks, governance reports
- not owned: authoritative runtime execution semantics
