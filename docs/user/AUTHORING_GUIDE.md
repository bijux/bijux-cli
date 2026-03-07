# DAG Authoring Guide

## Start with minimal JSON authoring
Use `tests/authoring/examples/minimal.json` as the baseline shape.

## Medium authoring example
Use `tests/authoring/examples/medium.json` for retries/resources/selectors structure.

## Validate and explain
Run `dag validate --explain <dag>` to get rule IDs and normalized context.

## Lint and canonicalize
- `dag graph-lint <dag>` for authoring hygiene warnings.
- `dag canonicalize <dag>` for stable normalized JSON.
- `dag show-effective-graph <dag>` for defaults + normalization view.
- `dag show-effective-plan <dag>` for execution lowering view.

## Node and edge naming guidance
- Use domain names (`extract_customers`) not generic names (`step1`).
- Keep IDs immutable once referenced in automation.
- Keep edge port names consistent with data shape purpose.

## Common patterns
- chain: `tests/authoring/examples/pattern_chain.json`
- diamond: `tests/authoring/examples/pattern_diamond.json`
- fanout: `tests/authoring/examples/pattern_fanout.json`
- aggregation: `tests/authoring/examples/pattern_aggregation.json`
- cache-heavy: `tests/authoring/examples/pattern_cache_heavy.json`
- replay-sensitive: `tests/authoring/examples/pattern_replay_sensitive.json`

## Common mistakes
- undeclared outputs: `tests/authoring/bad/undeclared_outputs.json`
- invalid refs: `tests/authoring/bad/invalid_refs.json`
- cycles: `tests/authoring/bad/cycle.json`
- invalid selectors: `tests/authoring/bad/invalid_selectors.json`
- unsupported adapter payload: `tests/authoring/bad/unsupported_adapter_payload.json`

## What this DAG tool intentionally does not do
- It does not provide a production-grade distributed controller in this repository.
- It does not provide YAML or DSL as first-class normative authoring surfaces.
- It does not auto-migrate arbitrary old/new DAG formats without explicit compatibility policy.
- It does not treat undeclared side effects as acceptable behavior.
