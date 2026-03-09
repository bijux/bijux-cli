# ERRORS FAILURES AND DIAGNOSTICS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/DIAGNOSTICS_MODES.md
# Diagnostics Modes

## Scope
Defines boundaries between default diagnostics and debug diagnostics.

## Modes
- `default`: user-facing summary with stable code/category and actionable hint when available.
- `debug`: includes internal context, source-chain data, and policy trace data.

## Boundaries
- Internal implementation details are hidden in default mode.
- Debug mode may include crate/module origin and underlying source errors.

## Related tests
- `crates/bijux-dag-app/tests/error_output_contract.rs`

## Versioning and change policy
New debug fields are additive. Removing existing default fields is breaking.

## SOURCE: docs/spec/ERROR_CODES.md
# Error Codes

This catalog tracks stable public code IDs.

## Taxonomy source
- Registry: `configs/policy/error_codes.json`
- Contract: `docs/spec/ERROR_CONTRACT.md`
- Taxonomy: `docs/spec/ERROR_TAXONOMY.md`

## Stable codes
- `BJX-PARSE-001` (`parse`) Input is not valid DAG JSON.
- `BJX-SCHEMA-001` (`schema`) JSON shape violates schema contract.
- `BJX-VALIDATION-001` (`validation`) Semantic graph validation failed.
- `BJX-CONFIG-001` (`config`) Invalid configuration input.
- `BJX-POLICY-001` (`policy`) Policy denied requested behavior.
- `BJX-EXEC-001` (`execution`) Node execution failed.
- `BJX-IO-001` (`io`) Filesystem or artifact I/O failed.
- `BJX-REPLAY-001` (`replay`) Replay mismatch against recorded artifacts.
- `BJX-CACHE-001` (`cache`) Cache contract or proof mismatch.
- `BJX-COMPAT-001` (`compatibility`) Compatibility contract violation.
- `BJX-INTERNAL-001` (`internal`) Unexpected internal failure path.

## Change policy
New public codes require:
1. Registry update.
2. Contract/reference docs update.
3. Error tests update.

## SOURCE: docs/spec/ERROR_CONTRACT.md
# Error Contract

## Scope
Defines error classes, stable machine-readable fields, and exit-code policy for user-facing failures.

## Error classes
- parse
- schema
- validation
- config
- policy
- execution
- io
- replay
- cache
- compatibility
- internal

## Invariants
- JSON output includes stable code and class fields.
- Human-readable output prioritizes exact cause and action guidance.
- Default diagnostics exclude internal debug context.
- Validation diagnostics include a deterministic `why this failed` section with rule IDs when available.
- Replay/cache mismatch diagnostics include previous-run comparison assist fields.

## Related tests
- `crates/bijux-dag-app/tests/output_contract.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/error_output_contract.rs`
- `crates/bijux-dag-app/tests/error_exit_contract.rs`

## Versioning and change policy
Public error code additions require docs plus test coverage in the same change.

## SOURCE: docs/spec/ERROR_TAXONOMY.md
# Error Taxonomy

## Scope
Defines the unified error category model across `bijux-dag-core`, `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`, and `bijux-dev-dag`.

## Categories
- `parse`
- `schema`
- `validation`
- `config`
- `policy`
- `execution`
- `io`
- `replay`
- `cache`
- `compatibility`
- `internal`

## Mapping intent
- Core parsing and structural checks map to `parse` and `schema`.
- Semantic DAG rules map to `validation`.
- Runtime policy denials map to `policy`.
- Adapter/process failures map to `execution`.
- Storage and filesystem failures map to `io`.
- Replay and cache contract mismatches map to `replay` and `cache`.

## Diagnostic ordering policy
User-facing diagnostic ordering is stable for deterministic inputs. New diagnostics append by stable sort key (`category`, `code`, `path`).

## Related tests
- `crates/bijux-dag-app/tests/error_output_contract.rs`
- `crates/bijux-dag-app/tests/error_exit_contract.rs`

## Versioning and change policy
Adding categories is breaking unless consumers are proven category-agnostic. Category meaning changes require docs and snapshot updates in the same change.

## SOURCE: docs/spec/FAILURE_TAXONOMY_CONTRACT.md
# Failure Taxonomy Contract

## Scope
This contract defines failure classes and recovery expectations for runtime, replay,
scheduler, adapter, and artifact integrity surfaces.

Authoritative code and tests:
- `crates/bijux-dag-runtime/src/runtime_core/governance/semantics.rs`
- `crates/bijux-dag-runtime/tests/runtime_failure_contracts.rs`
- `crates/bijux-dag-runtime/tests/runtime_recovery_contracts.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Failure classes
- `timeout`: execution exceeded declared time limits
- `cancelled`: explicit cancellation reached terminal state
- `dependency_failure`: upstream dependency failed or was unavailable
- `policy_violation`: execution violated policy constraints
- `cache_invalid`: cached evidence invalid for reuse
- `artifact_corruption`: artifact payload/proof integrity violation
- `adapter_failure`: backend/runtime infrastructure failure not mapped above

## Operational grouping
- transient candidates: timeout, adapter failure, selected dependency failures
- permanent candidates: policy violation, artifact corruption, structural dependency failures
- advisory diagnostic classes: replay mismatch and backend capability mismatch

## Recovery expectations
- checkpoint without terminal completion requires recovery action
- partial artifact presence requires recovery action
- recovery classification must be explicit for interruption scenarios:
  - process interruption
  - scheduler interruption
  - event stream corruption
  - bundle import interruption
  - backend communication interruption

## Explainability requirement
Failure-oriented operator surfaces must remain machine-readable and stable:
- `dag why-rerun`
- `dag run-explain-failure`
- replay mismatch reason grouping

## Benchmark requirement
Failure handling claims require benchmark evidence for:
- classification overhead and drift
- failure injection workflows
- recovery decision latency

## Stability level
Stable governance contract for `v0.1` release truth surfaces.

## SOURCE: docs/spec/SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md
# Superseded by system cluster contract

- Superseded by: [SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md](./SYSTEM_GUARANTEES_AND_INVARIANTS_CONTRACT.md)
- Appendix source: [appendices/system/SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md](./appendices/system/SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md)

## SOURCE: docs/spec/appendices/system/SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md
# System Health Diagnostics Contract

## Purpose

Define required health and diagnostics guarantees for system-level integrity, anomaly detection, and operator-facing verification workflows.

## Required command and diagnostics coverage

- system health check command surfaces
- artifact store health diagnostics and anomaly detection
- run history health diagnostics and anomaly detection
- runtime engine and scheduler health diagnostics
- adapter and backend capability health diagnostics
- bundle and replay integrity diagnostics
- diff and provenance integrity diagnostics
- artifact lineage diagnostics
- runtime telemetry inspection diagnostics
- determinism drift detection diagnostics

## Required governance artifacts

- system health regression corpus
- automated health verification suite definition
- health diagnostics documentation
- system health reporting dashboard
- health regression fixtures and summary report

## Required verification surfaces

- machine-readable corpus and suite contracts
- release-visible health reports under `docs/reports/foundation`
- completion contracts in `bijux-dev-dag`
