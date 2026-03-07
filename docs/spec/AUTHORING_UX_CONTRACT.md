# Authoring UX Contract

## Supported authoring surface
`bijux-dag` currently supports one authoritative authoring surface: JSON DAG files that conform to `spec: "0.1"`.

YAML/DSL/generated authoring are not normative product surfaces in this repository.

## Canonical examples
- Minimal executable DAG: `evidence/authoring/patterns/minimal.json`
- Medium executable DAG with retries/resources/selectors: `evidence/authoring/patterns/medium.json`

## Authoring evidence classification
- `minimal`: first-hour onboarding baseline.
- `patterns`: normative reusable graph structures.
- `negative`: normative invalid inputs bound to stable validation rule IDs.
- `examples`: illustrative end-to-end authoring samples.

Battle workflows under `evidence/battle/` are not allowed to be reused as authoring fixtures.

## Pattern examples
- chain: `evidence/authoring/patterns/pattern_chain.json`
- diamond: `evidence/authoring/patterns/pattern_diamond.json`
- fanout: `evidence/authoring/patterns/pattern_fanout.json`
- aggregation: `evidence/authoring/patterns/pattern_aggregation.json`
- cache-heavy: `evidence/authoring/patterns/pattern_cache_heavy.json`
- replay-sensitive: `evidence/authoring/patterns/pattern_replay_sensitive.json`

## Common mistake fixtures
- undeclared outputs: `evidence/authoring/negative/undeclared_outputs.json`
- invalid refs: `evidence/authoring/negative/invalid_refs.json`
- cycles: `evidence/authoring/negative/cycle.json`
- invalid selectors: `evidence/authoring/negative/invalid_selectors.json`
- unsupported adapter payload: `evidence/authoring/negative/unsupported_adapter_payload.json`

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
Examples in user-facing docs must reference executable fixture files under `evidence/authoring/`.
Hand-maintained prose-only DAG snippets are not allowed as normative examples.

## Intentionally out of scope
See `docs/user/AUTHORING_GUIDE.md` section `What this DAG tool intentionally does not do`.
