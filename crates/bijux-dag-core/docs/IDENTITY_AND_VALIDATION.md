# Identity And Validation

Only well-defined graph semantics can produce trustworthy identity or planner
input. Validation and identity must not depend on source formatting, map
insertion order, or machine state.

## Validation Pipeline

`parse_graph_strict` rejects unknown fields and malformed shape. Resolution and
validation then check specification compatibility, identifiers, references,
nodes, edges, inputs, outputs, resources, topology, branches, trigger rules,
templates, path variables, subgraphs, and dynamic expansion.

Diagnostics retain a stable code, severity, path, and actionable context where
available. Diagnostic ordering is deterministic.

## Canonicalization

`canonicalize_graph` and `canonical_json` remove representation differences
while retaining semantic differences. Equivalent maps and authored ordering
produce the same canonical bytes; execution-relevant mutations produce
different bytes.

`CANONICALIZATION_CONTRACT_VERSION` identifies the algorithm contract.
Changing canonical output is a compatibility event even when `Graph` still
deserializes.

## Fingerprints

Graph fingerprints hash canonical bytes. Node and planner identities use their
governed semantic factors. Identity explanations expose canonical input, byte
length, and algorithm so mismatches can be diagnosed.

Identity excludes source paths, wall-clock time, undeclared host environment,
collection iteration order, and non-semantic presentation differences. It
changes when governing execution inputs, dependencies, outputs, resources,
effects, branches, or policies change.

## Change Review

Changes to graph shape, defaults, canonicalization, or identity require:

- round-trip and snapshot review;
- positive and refusal fixtures;
- identity mutation tests;
- downstream cache and replay impact assessment;
- an explicit stable compatibility decision.

Do not refresh snapshots mechanically until every changed byte has a semantic
explanation.

## Verification

```bash
cargo test --locked -p bijux-dag-core --test canonical_contract
cargo test --locked -p bijux-dag-core --test graph_identity_contract
cargo test --locked -p bijux-dag-core --test identity_mutation_contracts
cargo test --locked -p bijux-dag-core --test validation_entrypoints_contract
```

Property, fuzz, adversarial, fixture, and scale contracts cover broader input
classes and ordering laws.
