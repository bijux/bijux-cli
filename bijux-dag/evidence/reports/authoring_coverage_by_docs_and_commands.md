# Authoring Coverage by Docs and Commands

| Asset | Referenced in docs | Command surfaces |
| --- | --- | --- |
| `evidence/authoring/examples/cached-branched-report.dag.json` | no | `dag validate, dag run, dag show-effective-plan` |
| `evidence/authoring/examples/etl-constant-to-shell.dag.json` | no | `dag validate, dag run, dag show-effective-plan` |
| `evidence/authoring/examples/failure-heavy-retry.dag.json` | no | `dag validate, dag run, dag show-effective-plan` |
| `evidence/authoring/examples/hello.dag.json` | no | `dag validate, dag run, dag show-effective-plan` |
| `evidence/authoring/examples/multi-output-artifact.dag.json` | no | `dag validate, dag run, dag show-effective-plan` |
| `evidence/authoring/examples/replay-heavy-branching.dag.json` | no | `dag validate, dag run, dag show-effective-plan` |
| `evidence/authoring/negative/cycle.json` | yes | `dag validate, dag validate --explain` |
| `evidence/authoring/negative/invalid_refs.json` | yes | `dag validate, dag validate --explain` |
| `evidence/authoring/negative/invalid_selectors.json` | yes | `dag validate, dag validate --explain` |
| `evidence/authoring/negative/undeclared_outputs.json` | yes | `dag validate, dag validate --explain` |
| `evidence/authoring/negative/unsupported_adapter_payload.json` | yes | `dag validate, dag validate --explain` |
| `evidence/authoring/patterns/medium.json` | yes | `dag validate, dag graph-lint, dag show-effective-plan` |
| `evidence/authoring/patterns/minimal.json` | yes | `dag validate, dag show-effective-graph, dag show-effective-plan` |
| `evidence/authoring/patterns/pattern_aggregation.json` | yes | `dag validate, dag show-effective-plan` |
| `evidence/authoring/patterns/pattern_cache_heavy.json` | yes | `dag validate, dag show-effective-plan` |
| `evidence/authoring/patterns/pattern_chain.json` | yes | `dag validate, dag show-effective-plan` |
| `evidence/authoring/patterns/pattern_diamond.json` | yes | `dag validate, dag show-effective-plan` |
| `evidence/authoring/patterns/pattern_fanout.json` | yes | `dag validate, dag show-effective-plan` |
| `evidence/authoring/patterns/pattern_replay_sensitive.json` | yes | `dag validate, dag show-effective-plan` |
