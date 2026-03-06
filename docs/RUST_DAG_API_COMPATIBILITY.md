# Rust DAG API compatibility promise

## Scope

The Rust DAG authoring API exposed from `bijux-dag-core` is intended for external consumers.

## Compatibility commitment

- Public builder and DSL types use semantic version compatibility.
- Breaking changes require a major version increment and migration notes.
- New fields may be added with defaults; removals and incompatible type changes are breaking.

## Covered API surface

- `DagBuilder`
- `NodeBuilder`
- lint and dry-run preview contracts
- simulation and unit harness interfaces

## Exclusions

- internal helper functions not exported from crate root
- unstable experimental modules not listed in crate public exports
