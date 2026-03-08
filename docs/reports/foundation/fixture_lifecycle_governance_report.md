# Fixture Lifecycle Governance Report

## Lifecycle stages

1. generate fixture payloads and scenario assets
2. validate fixtures against schema and parsing contracts
3. enforce deterministic and portable fixture behavior
4. detect duplication and stale ownership signals
5. clean up stale or unreferenced fixtures

## Governance anchors

- policy: `configs/policy/fixture_family_governance.json`
- contract: `docs/spec/FIXTURE_TOOLING_GOVERNANCE_CONTRACT.md`
- suite: `configs/suites/fixture_tooling_governance.json`
- corpus: `evidence/cache/fixture_tooling/regression_corpus.json`
