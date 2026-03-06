# Task contract type system

## Type vocabulary

Task contracts use a formal type registry:

- scalar types
- collection types
- versioned serialization rules
- explicit coercion rules

Silent coercion is forbidden. Coercion must be declared and compatibility-scoped.

## Contract semantics

- nullability, optionality, cardinality
- secret references (distinct from normal strings/env)
- resource references for datasets, models, or prior artifacts
- partitioned collection contracts
- bounded polymorphic variants

## Adapter and replay compatibility

- Adapter capability declarations are checked against type requirements.
- Replay compatibility checks validate adapter version alignment and declared support.

## Validation and diagnostics

- Parameter default compatibility validation
- Cross-node producer/consumer contract validation
- Path-level diagnostics for mismatch locations

## Fingerprints and evolution

- Task contract fingerprints are separate from node fingerprints.
- Output evolution markers include backward and forward compatibility flags.

## Generated documentation and matrix reporting

- Contract markdown can be generated directly from typed contract structures.
- Compatibility matrix reports summarize producer-consumer relationships across a DAG snapshot.
