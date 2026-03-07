# Authoring UX Contract

## Supported authoring surface
`bijux-dag` currently supports one authoritative authoring surface: JSON DAG files that conform to `spec: "0.1"`.

YAML/DSL/generated authoring are not normative product surfaces in this repository.

## Canonical examples
- Minimal executable DAG: `tests/authoring/examples/minimal.json`
- Medium executable DAG with retries/resources/selectors: `tests/authoring/examples/medium.json`

## Pattern examples
- chain: `tests/authoring/examples/pattern_chain.json`
- diamond: `tests/authoring/examples/pattern_diamond.json`
- fanout: `tests/authoring/examples/pattern_fanout.json`
- aggregation: `tests/authoring/examples/pattern_aggregation.json`
- cache-heavy: `tests/authoring/examples/pattern_cache_heavy.json`
- replay-sensitive: `tests/authoring/examples/pattern_replay_sensitive.json`

## Common mistake fixtures
- undeclared outputs: `tests/authoring/bad/undeclared_outputs.json`
- invalid refs: `tests/authoring/bad/invalid_refs.json`
- cycles: `tests/authoring/bad/cycle.json`
- invalid selectors: `tests/authoring/bad/invalid_selectors.json`
- unsupported adapter payload: `tests/authoring/bad/unsupported_adapter_payload.json`

## Authoring command surfaces
- `dag validate --explain <dag>`
- `dag graph-lint <dag>`
- `dag canonicalize <dag>`
- `dag show-effective-graph <dag>`
- `dag show-effective-plan <dag>`

## Naming guidance
- Node IDs must be unique, stable, and domain-specific.
- IDs must avoid ambiguous short aliases.
- Edge references must target existing node IDs and declared ports.
- Guidance is tied to validation rules documented in `docs/spec/VALIDATION_RULES.md`.

## Documentation and fixture rule
Examples in user-facing docs must reference executable fixture files under `tests/authoring/`.
Hand-maintained prose-only DAG snippets are not allowed as normative examples.

## Intentionally out of scope
See `docs/user/AUTHORING_GUIDE.md` section `What this DAG tool intentionally does not do`.
