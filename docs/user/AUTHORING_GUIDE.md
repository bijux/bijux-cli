# DAG Authoring Guide

Audience: DAG authors and operators.  
Owner: user documentation maintainers.  
Status: stable.

## Start with minimal JSON authoring
Use `evidence/authoring/patterns/minimal.json` as the baseline shape.

## Medium authoring example
Use `evidence/authoring/patterns/medium.json` for retries/resources/selectors structure.

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
- chain: `evidence/authoring/patterns/pattern_chain.json`
- diamond: `evidence/authoring/patterns/pattern_diamond.json`
- fanout: `evidence/authoring/patterns/pattern_fanout.json`
- aggregation: `evidence/authoring/patterns/pattern_aggregation.json`
- cache-heavy: `evidence/authoring/patterns/pattern_cache_heavy.json`
- replay-sensitive: `evidence/authoring/patterns/pattern_replay_sensitive.json`

## Common mistakes
- undeclared outputs: `evidence/authoring/negative/undeclared_outputs.json`
- invalid refs: `evidence/authoring/negative/invalid_refs.json`
- cycles: `evidence/authoring/negative/cycle.json`
- invalid selectors: `evidence/authoring/negative/invalid_selectors.json`
- unsupported adapter payload: `evidence/authoring/negative/unsupported_adapter_payload.json`

## What this DAG tool intentionally does not do
- It does not provide a production-grade distributed controller in this repository.
- It does not provide YAML or DSL as first-class normative authoring surfaces.
- It does not auto-migrate arbitrary old/new DAG formats without explicit compatibility policy.
- It does not treat undeclared side effects as acceptable behavior.
