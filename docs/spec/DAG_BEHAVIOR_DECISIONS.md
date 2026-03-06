# DAG behavior decisions

## nondeterminism_allowed
`nondeterminism_allowed` is a transitional compatibility flag.

Permitted when enabled:
- Retry behavior for nodes that use clock/network effects without explicit deterministic seed input.

Never permitted even when enabled:
- Skipping validation of graph shape and reference integrity.
- Accepting invalid output paths.
- Accepting invalid node/tag/graph identifiers.

## group semantics
`group` is annotation only in v0.1.
It is not scheduling input and is excluded from node fingerprints.

## Node inputs and edge references
`inputs: Vec<String>` remains explicit and non-redundant in v0.1.
It defines node input interface contracts; edges bind dataflow to that interface.

## container payload
`container` remains a node-kind-specific payload in v0.1.
A typed cross-kind payload enum is deferred until a v0.2 compatibility window, to avoid breaking adapter contracts during boundary refactoring.
