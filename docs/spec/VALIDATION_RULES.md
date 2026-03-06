# Validation Rules

Validation rule registry source: `crates/bijux-dag-core/src/validate.rs`.

## Validation domains
- `Schema`: structural and shape constraints.
- `Semantic`: behavior and meaning constraints.
- `Topology`: graph connectivity and ordering constraints.

## Error Codes
- `E1001` Duplicate node id
- `E1002` Dangling node reference
- `E1003` Dangling port reference
- `E1004` Cycle detected
- `E1005` JSON parse error / unknown fields
- `E1006` Invalid spec version
- `E1007` Illegal node id characters
- `E1008` Output collision
- `E1009` Missing effects declaration
- `E1010` Env allowlist without env effect
- `E1011` Retry disallowed for nondeterministic effects
- `E1013` Effect denied by policy (network/env/clock)
- `E1020` Unknown graph input reference
- `E1021` Unknown node output reference
- `E1022` Forward node output reference
- `E1023` Missing container spec for container node
- `E1024` Invalid container spec
- `E1025` Invalid output file path

## Warning Codes
- `W2001` Unreachable node
- `W2002` Orphan node

## Rules
1. Node ids must be unique. (`E1001`)
2. Edge node references must exist. (`E1002`)
3. Edge port references must exist on their nodes. (`E1003`)
4. The graph must be acyclic. (`E1004`)
5. JSON must be strict with no unknown fields. (`E1005`)
6. DAG spec version must be known. (`E1006`)
7. Node ids must match `[a-zA-Z0-9_-]+`. (`E1007`)
8. Output names must be unique across nodes. (`E1008`)
9. Shell nodes must declare effects and include filesystem. (`E1009`)
10. env_allowlist requires env effect. (`E1010`)
11. Retry with clock/network requires random_seed or nondeterminism_allowed. (`E1011`)
12. Parameter references must be valid graph inputs or node outputs. (`E1020`, `E1021`, `E1022`)
13. Effect denied by policy when `--deny-network`, `--deny-env`, or `--deny-clock` used. (`E1013`)
14. Graph input ref must exist. (`E1020`)
15. Node output ref must exist. (`E1021`)
16. Node output ref must not point to downstream node. (`E1022`)
17. Container nodes must include a container spec. (`E1023`)
18. Container spec must be valid (engine and argv). (`E1024`)
19. Output file paths must be relative and not contain `..`. (`E1025`)
20. Nodes not reachable from any root emit a warning. (`W2001`)
21. Nodes with no edges emit a warning. (`W2002`)
