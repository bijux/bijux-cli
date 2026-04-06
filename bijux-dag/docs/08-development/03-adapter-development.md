# Adapter Development

This guide describes how to build a backend adapter without redefining runtime semantics.

## Adapter contract

An adapter must:
- execute normalized work units,
- map backend-native outcomes to canonical runtime outcomes,
- emit evidence required for inspect/replay/diff,
- declare capability limits explicitly.

An adapter may decide:
- backend invocation mechanics,
- backend-specific diagnostics collection,
- operational tuning inside declared capability envelope.

An adapter must never redefine:
- DAG semantics,
- run/artifact identity rules,
- replay/diff classification vocabulary.

## Adapter lifecycle

1. declare capability model and support class.
2. implement execution and normalization mapping.
3. implement artifact and lineage emission.
4. validate with integration and end-to-end fixtures.
5. document degradation behavior and maintenance ownership.

## End-to-end worked example

Example: `container-x` adapter.

1. Capability declaration:
- supports timeout: yes
- supports cancel: yes
- supports streaming artifact upload: no
- support class: bounded

2. Execution mapping:
- backend exit `0` -> `succeeded`
- backend timeout signal -> `failed_timeout`
- non-zero exit -> `failed_non_zero`

3. Evidence emission:
- attach `run_id` and `node_id` to produced artifact records,
- persist backend diagnostic fields in normalized error envelope.

4. Validation workflow:
- run fixture set against stable `local-shell` baseline,
- run replay/diff comparison for required scopes,
- classify differences as equivalent/drift/incomplete with reason codes.

5. Publish limitations:
- streaming unsupported; artifact buffering required,
- strict replay not guaranteed for streaming-dependent workloads.

## Failure cases adapters must handle explicitly

- timeout/cancel ambiguity from backend API,
- partial artifact write with missing lineage link,
- backend error payload that lacks deterministic cause,
- capability mismatch requested by runtime plan.

## Guarantees

- Adapter responsibilities are explicit and enforceable.
- Capability limitations are surfaced as contract input.

## Non-guarantees

- Feature parity with every backend family.
- Strict equivalence outside declared capability envelope.

## Next reading

- [Adapters architecture](docs/05-system-architecture/05-adapters.md)
- [Backend support](docs/07-operations/05-backend-support.md)
- [Run model contract](docs/06-specification/02-run-model.md)
