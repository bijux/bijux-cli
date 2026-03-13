# Decision Rules

## Purpose
This document defines the architectural rules that preserve bijux-cli invariants. It exists to prevent accidental design drift as the codebase evolves.

## Scope
It covers decision ownership and forbidden behaviors. It does not specify implementation details or coding style.

## Core Rules
Decisions about policy, output routing, and exit behavior must occur in core. Infra components must execute explicit instructions and never guess or normalize values.
Runtime identity is singular: `bijux` is the only canonical user-facing runtime binary name, and all supported entrypoints must execute the same law.

## Why These Rules Exist
These rules prevent late overrides and inconsistent behavior across different entry points. They are enforced by tests and are considered part of the public contract.

## Failure Modes
When rules are violated, behavior becomes nondeterministic and tests may pass locally while failing in automation. These violations are treated as regressions and must be corrected, not documented away.

## Design Rationale
By centralizing decisions, we make the system predictable and testable. This also makes it easier to reason about failure handling and exit codes.

## Non-Goals
This document does not define project process or governance.

## Runtime Law Freeze
The single canonical runtime identity rule is non-negotiable and frozen until an explicit breaking-change policy replaces it.
New maintainer automation defaults to `bijux dev cli` commands, not ad hoc scripts.

## Docs Rule Freeze
Documentation stays intentionally small. Each long-form document must explain law or explain change, and low-value detail should move into generated artifacts or snapshots.

## Test Rule Freeze
No vanity test counts. Quality claims require evidence from failure-path, exit-code, output-regression, and resilience coverage.
Config mutation hardening must remain evidenced by:
`artifacts/status/config_corruption_matrix.json` and
`artifacts/status/config_rollback_proof.json`.

## Parity Report Freeze
Every release candidate must include command-level parity matrix and diff artifacts.

Parity governance remains artifact-backed:

- `artifacts/parity/command_parity_matrix.json`
- `artifacts/parity/command_parity_diffs.json`
- `artifacts/parity/parity_regression_diffs.json`
- `docs/architecture/parity/intentional_differences.json`

No parity-covered command may regress from `complete` to `partial` or `missing`
without updating the matrix and diffs in the same change.

## Plugin V1 Freeze
Plugin v1 behavior is frozen before introducing new plugin command complexity beyond parity-backed scope.
Write-path hardening evidence must remain green:
`artifacts/status/plugin_lifecycle_failure_injection_report.json` and
`artifacts/status/plugin_rollback_proof_report.json`.

## State Resilience Freeze
History and memory resilience claims require:
`artifacts/status/history_corruption_matrix.json`,
`artifacts/status/memory_corruption_matrix.json`, and
`artifacts/status/state_resilience_summary.json`.

## REPL Recovery Freeze
REPL resilience claims require:
`artifacts/status/repl_hostile_session_report.json` and
`artifacts/status/repl_recovery_behavior_report.json`.

## Install Ambiguity Freeze
Install-neutrality claims require:
`artifacts/status/packaging_ambiguity_report.json`,
`artifacts/status/install_state_assumptions_report.json`, and
`artifacts/status/package_health_report.json`.

## Performance Budget Freeze
Performance claims require:
`artifacts/status/performance_report.json`,
`artifacts/status/performance_regression_budget.json`, and
`artifacts/status/performance_benchmark_policy.json`.
Only critical-path command regressions are release-gated.

## Contributor Status Rule
Contributors describe observed reality in status updates using generated artifacts. Avoid aspirational language that is not yet evidenced.

## Maintainer Evidence Rule
Every status update must include explicit `done`, `left`, and `blocked/deferred` lists backed by artifact paths.

## Major Command Rule
Each major command area requires a parity report before non-parity improvements are accepted.

## Public Claim Rule
Any public claim in README must be evidence-backed by repository artifacts or generated reports.

## Release Evidence Freeze
Release claims require:
`artifacts/status/release_evidence_bundle.json`,
`artifacts/status/release_status_manifest.json`, and
`artifacts/status/release_truth_report.json`.
Migration guidance must be generated from artifacts, not hand-curated prose.

## Compatibility Shim Freeze
Compatibility shims and aliases require generated inventory with justification and removal plans:
`artifacts/status/compatibility_shim_inventory.json` and
`artifacts/status/compatibility_alias_inventory.json`.
Permanent compatibility shims are rejected by policy gate unless evidence explicitly justifies them.

## Parser Hardening Freeze

Parser and routing behavior must stay deterministic under malformed argv,
ambiguity, alias pressure, and registration-order variation.
Reserved and built-in namespaces must not be hijacked by plugin namespaces.

Evidence:

- `crates/bijux-cli/tests/routing/parser/parser_abuse.rs`
- `artifacts/status/parser_abuse_report.json`

## Rustdoc Ownership Freeze

Rustdoc is the primary code documentation path for `bijux-cli`.

- public Rust APIs must be documented in Rust source
- website and API docs should link to Rustdoc-owned truth instead of duplicating
  API semantics
- `bijux dev cli rustdoc audit` is the maintainer-facing authority for Rustdoc
  health
- release claims must not describe code documentation as complete without
  matching Rustdoc audit artifacts

## Truth Before Polish Freeze
Truth-reporting and parity evidence gates are frozen as release requirements before polish-only work.
