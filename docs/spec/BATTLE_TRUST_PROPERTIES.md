# Battle trust properties

## Scope

This document defines the canonical trust properties for battle workflow evidence.

## Canonical trust properties

- `tp_deterministic_scheduling`
- `tp_failure_propagation`
- `tp_replay_equivalence`
- `tp_cache_integrity`
- `tp_artifact_integrity`
- `tp_policy_enforcement`
- `tp_operator_observability`
- `tp_import_export_compatibility`
- `tp_state_machine_legality`
- `tp_timeout_retry_accounting`
- `tp_secret_redaction`
- `tp_run_dir_resilience`

## Authority and mapping

The normative source for trust property metadata and scenario mapping is [`configs/policy/battle_trust_properties.json`](../../configs/policy/battle_trust_properties.json).

## Governance rules

- Every battle workflow scenario must map to one or more canonical trust properties.
- No battle scenario is admitted without an owner and a `why_exists` statement.
- Drift checks must reject orphan scenarios and unknown trust property identifiers.
- Foundation suite execution includes battle trust coverage checks.
