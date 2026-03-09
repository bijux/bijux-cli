# EVIDENCE MODEL AND PUBLICATION

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/AUDIT_REPORT_CONTRACT.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/AUDIT_REPORT_CONTRACT.md](./appendices/evidence/AUDIT_REPORT_CONTRACT.md)

## SOURCE: docs/spec/BENCHMARK_EVIDENCE_AND_CLAIM_CONTRACT.md
# Benchmark evidence and claim contract

**What this spec is not**: runtime implementation plan, benchmark tooling internals, or release process.

## Scope

This contract defines:

- benchmark claim classes and publication criteria
- scenario registry and reproducibility requirements
- result format requirements and retention expectations
- minimal evidence model for performance claims

## Canonical requirements

- All published performance claims must point to raw benchmark data and scenario metadata.
- Benchmarks are classified by claim class and threshold policy before publication.
- Raw benchmark artifacts are retained for the evidence link chain and may only be compacted with replacement baselines.
- Scenario contracts are the single source for scenario identity and meaning.

## Evidence and implementation links

- Evidence schema: `configs/schema/benchmarks/benchmark_report.schema.json`
- Benchmark suites and comparisons in `crates/bijux-dev-dag`.
- Evidence policy: `configs/policy/benchmark_signal_gov...` and related governance artifacts.

## Canonical appendices

- [reproducibility](./appendices/benchmark/BENCHMARK_REPRODUCIBILITY_CONTRACT.md)
- [result format](./appendices/benchmark/BENCHMARK_RESULT_FORMAT.md)
- [scenario contract](./appendices/benchmark/BENCHMARK_SCENARIO_CONTRACT.md)
- [types](./appendices/benchmark/BENCHMARK_TYPES.md)
- [scorecard guide](./appendices/benchmark/BENCHMARK_SCORECARD_GUIDE.md)
- [minimalism policy](./appendices/benchmark/BENCHMARK_MINIMALISM_POLICY.md)
- [raw data retention](./appendices/benchmark/BENCHMARK_RAW_DATA_RETENTION.md)

## SOURCE: docs/spec/DETERMINISM_EVIDENCE_CONTRACT.md
# Determinism Evidence Contract

Determinism evidence is complete only when run verification passes without errors and invariant violations.

Evidence levels:
- `verified`
- `insufficient-evidence`

## SOURCE: docs/spec/EVIDENCE_GLOSSARY.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/EVIDENCE_GLOSSARY.md](./appendices/evidence/EVIDENCE_GLOSSARY.md)

## SOURCE: docs/spec/EVIDENCE_INTERNAL_ONLY_SURFACES.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/EVIDENCE_INTERNAL_ONLY_SURFACES.md](./appendices/evidence/EVIDENCE_INTERNAL_ONLY_SURFACES.md)

## SOURCE: docs/spec/EVIDENCE_MODEL.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/EVIDENCE_MODEL.md](./appendices/evidence/EVIDENCE_MODEL.md)

## SOURCE: docs/spec/EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md
# Evidence model, publication, and governance contract

**What this spec is not**: benchmark execution playbooks, release engineering procedure, or implementation internals.

## Scope

This contract is the canonical source for:

- evidence vocabularies and trust claims
- evidence publication and release-claim gating
- internal-only evidence boundaries
- audit report and registry access contracts

## Canonical principles

- Evidence claims require reproducible artifacts and traceable source links.
- Internal diagnostic surfaces are not suitable as release evidence.
- Vocabulary consistency is required across tests, docs, and governance surfaces.
- Audit findings must differentiate unsupported approximations from implemented guarantees.

## Canonical evidentiary model

Refer to appendix sections for:

- model terms and glossary
- publication quality and trust lanes
- internal-only and public evidence separation
- audit contract and access controls

## Implementation and evidence sources

- Evidence registry and fixtures under `evidence/`.
- Verification workflows in `crates/bijux-dev-dag`.
- Evidence contracts and testkit access surfaces in `crates/bijux-dag-testkit`.

## SOURCE: docs/spec/EVIDENCE_PUBLICATION_CONTRACT.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/EVIDENCE_PUBLICATION_CONTRACT.md](./appendices/evidence/EVIDENCE_PUBLICATION_CONTRACT.md)

## SOURCE: docs/spec/EVIDENCE_RELEASE_NOTE_TRUST.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/EVIDENCE_RELEASE_NOTE_TRUST.md](./appendices/evidence/EVIDENCE_RELEASE_NOTE_TRUST.md)

## SOURCE: docs/spec/EVIDENCE_TERMS_AND_GOVERNANCE.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/EVIDENCE_TERMS_AND_GOVERNANCE.md](./appendices/evidence/EVIDENCE_TERMS_AND_GOVERNANCE.md)

## SOURCE: docs/spec/INTEGRITY_EVIDENCE_CONTRACT.md
# Integrity Evidence Contract

Integrity evidence is complete only when output index and artifact hashes verify successfully.

Evidence levels:
- `verified`
- `insufficient-evidence`

## SOURCE: docs/spec/PROOF_BUNDLE_CONTRACT.md
# Proof Bundle Contract

Defines the machine-readable proof bundle generated by `bijux dag prove <run-dir>`.

Required fields:
- `schema_version`
- `proof_id`
- `run_id`
- `status`
- `complete`
- `determinism`
- `integrity`
- `replay_evidence`
- `integrity_evidence`
- `incomplete_reasons`
- `signing`

## SOURCE: docs/spec/TESTKIT_EVIDENCE_ACCESS_CONTRACT.md
# Superseded by evidence cluster contract

- Superseded by: [EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md](./EVIDENCE_MODEL_AND_PUBLICATION_CONTRACT.md)
- Appendix source: [appendices/evidence/TESTKIT_EVIDENCE_ACCESS_CONTRACT.md](./appendices/evidence/TESTKIT_EVIDENCE_ACCESS_CONTRACT.md)

## SOURCE: docs/spec/TEST_EVIDENCE_CONSUMER_CONTRACT.md
# Test Evidence Consumer Contract

## Rule

Test code is a consumer of governed evidence assets. Test code does not define canonical scenario truth.
Canonical terms are defined in `docs/spec/EVIDENCE_GLOSSARY.md`.
Evidence consumers must resolve assets through typed access helpers, not direct filesystem reads of the registry.
Evidence consumption is read-only; tests must not mutate files under `evidence/`.

## Canonical ownership

- Authoring examples: `evidence/authoring/**`
- Battle scenarios: `evidence/battle/**`
- Cache and replay scenarios: `evidence/cache/**`
- Compatibility scenarios: `evidence/compat/**`
- Fault scenarios: `evidence/fault/**`
- Performance scenarios and baselines: `evidence/perf/**`
- Comparison scenarios and baselines: `evidence/compare/**`

## Forbidden ownership patterns

- Scenario assets under `tests/e2e/fixtures/**`
- Scenario assets under `tests/e2e/replay/fixtures/**`
- Scenario assets under `tests/e2e/compat/**`
- Scenario assets under `tests/e2e/container/**`
- Scenario assets under `benchmarks/scenarios/**`
- Scenario assets under `comparisons/scenarios/**`

## Enforcement surfaces

- `bijux-dev-dag verify evidence-ownership`
- `bijux-dev-dag verify evidence-drift`
- `bijux-dev-dag verify evidence-consumers`
- `bijux-dev-dag repo evidence-resolve-by-id --id <asset-id>`
- `bijux-dev-dag repo evidence-resolve-by-family --family <family>`
- `bijux-dev-dag repo evidence-resolve-by-trust-property --trust-property <trust-id>`
- `bijux-dev-dag repo evidence-resolve-by-consumer --consumer <consumer-id>`
- `bijux-dev-dag` contract and test suite id: `evidence-consumer-integrity`
- `bijux-dag-testkit` evidence access helpers:
  - `load_evidence_registry_checked`
  - `resolve_evidence_asset_by_id_checked`
  - `evidence_asset_ids`

## Consumer mapping reports

- `evidence/reports/evidence_consumers_inventory.md`
- `evidence/reports/evidence_assets_to_consumers.md`
- `evidence/reports/evidence_consumers_to_families.md`
- `evidence/reports/evidence_consumption_by_crate.md`

## SOURCE: docs/spec/appendices/evidence/AUDIT_REPORT_CONTRACT.md
# Audit Report Contract

Audit reports summarize unsupported capability approximations, simulated surfaces, and API evolution impacts.

Required sections:
- unsupported capability approximations
- simulated versus implemented features
- public API surface changes

## SOURCE: docs/spec/appendices/evidence/EVIDENCE_GLOSSARY.md
# Evidence Glossary

- `evidence`: committed, executable, and traceable artifacts used to support trust claims.
- `proof`: deterministic conclusion derived from verification outputs over evidence assets.
- `verification`: executable checks that validate policy, integrity, and consistency constraints.
- `governance`: repository rules that define ownership, blocking behavior, and release eligibility.
- `release-critical evidence`: evidence that must pass in the full lane before release.
- `advisory evidence`: evidence surfaced for operator insight and trend tracking, non-blocking by default.

## SOURCE: docs/spec/appendices/evidence/EVIDENCE_INTERNAL_ONLY_SURFACES.md
# Internal-Only Evidence Surfaces

The following outputs are internal diagnostics and should not be used as release-note claim sources:
- `docs/reports/foundation/EVIDENCE_CI_EXERCISE_REPORT.md`
- `docs/reports/foundation/evidence_report_consolidation.md`
- `docs/reports/foundation/evidence_docs_without_checks_report.md`
- `docs/reports/foundation/evidence_checks_without_docs_report.md`

These surfaces support governance operations and gap analysis, not public release claims.

## SOURCE: docs/spec/appendices/evidence/EVIDENCE_MODEL.md
# Evidence model

This file defines what counts as proof.

## Correctness evidence

- Deterministic contract tests
- Negative fixtures for rejected behavior
- Invariant checks and diagnostics snapshots

## Compatibility evidence

- Versioned schema fixtures
- Compatibility matrix fixtures
- Migration and downgrade checks

## Performance evidence

- Measured benchmark artifacts produced by benchmark suites
- Trend reports with workload metadata

`benchmark-baseline` must carry workload metadata and environment context before it is used
as release proof.

## Memory evidence

- Measured memory artifacts with explicit environment metadata
- Budget regression checks

Memory evidence must come from measured benchmark/observability artifacts; standalone smoke
timing is not accepted as release proof.

## Release readiness evidence

- Contract suite report
- Compatibility report
- Security and policy checks
- Rollback and migration verification artifacts

## Guarantee language rule

Any guarantee statement in repository docs must include a markdown link to proof in one of:
- `docs/spec/`
- test fixture/test file
- benchmark artifact/report

## SOURCE: docs/spec/appendices/evidence/EVIDENCE_PUBLICATION_CONTRACT.md
# Evidence Publication Contract

## Purpose
Define publication rules for benchmark evidence used in docs, release notes, and comparisons.
Canonical terms are defined in `docs/spec/EVIDENCE_GLOSSARY.md`.

## Evidence publication rules
- Performance claims in docs must point to committed benchmark artifacts.
- Published evidence must include both raw benchmark report data and scenario metadata.
- Generated summaries may be used only when raw reports are still available.
- Evidence with missing scenario registry links is non-compliant.

## Required publication surfaces
- Scenario definitions: `evidence/perf/scenarios/`
- Scenario registry: `evidence/perf/scenario_registry.json`
- Metadata policy: `evidence/perf/metadata.json`
- Baselines and thresholds: `evidence/perf/baselines/`

## Governance
- `docs/spec/PERFORMANCE_CONTRACT.md` remains the top-level claim policy.
- This contract governs publication quality and traceability for benchmark evidence.

## SOURCE: docs/spec/appendices/evidence/EVIDENCE_RELEASE_NOTE_TRUST.md
# Evidence Trust for Release Notes

Release-note-safe evidence outputs:
- `evidence/release/release_evidence.json`
- `evidence/reports/what_this_release_proves.md`
- `evidence/reports/what_this_release_does_not_prove.md`
- `docs/reports/foundation/RELEASE_CRITICAL_EVIDENCE_MATRIX.md`

These outputs are suitable for release-note claims because they are tied to executable verification surfaces and lane gates.

## SOURCE: docs/spec/appendices/evidence/EVIDENCE_TERMS_AND_GOVERNANCE.md
# Evidence, Proof, Verification, and Governance

Canonical vocabulary and meanings are defined in:
- `docs/spec/EVIDENCE_GLOSSARY.md`

Operational model:
1. Evidence assets are stored under governed roots and tracked by the ledger/registry.
2. Verification commands (`bijux-dev-dag verify evidence-*`) evaluate policy and integrity contracts.
3. Proof surfaces aggregate verification outcomes for release and trust communication.
4. Governance policy classifies which evidence checks are release-critical versus advisory.

Lane behavior:
- Fast lane keeps advisory evidence non-blocking by default.
- Full lane executes the release-critical evidence command set and blocks on failure.

## SOURCE: docs/spec/appendices/evidence/TESTKIT_EVIDENCE_ACCESS_CONTRACT.md
# Testkit Evidence Access Contract

## Purpose

`bijux-dag-testkit` provides the shared read-only access boundary for governed evidence assets in tests.

## Access helpers

- `evidence_registry_path(workspace_root)` returns the canonical registry location.
- `load_evidence_registry(workspace_root)` loads the registry and panics on malformed state.
- `load_evidence_registry_checked(workspace_root)` returns actionable read/parse errors with the registry path.
- `resolve_evidence_asset_by_id(registry, id)` resolves one asset and panics if missing.
- `resolve_evidence_asset_by_id_checked(registry, id)` returns actionable diagnostics for missing ids.
- `evidence_asset_ids(registry)` returns stable sorted asset IDs for reload-drift checks.

## Rules

- Helpers are read-only. They never mutate files under `evidence/`.
- Tests must resolve canonical assets by id through these helpers instead of hand-wired filesystem crawling.
- Missing assets must return diagnostics that include:
  - the missing asset id
  - a next-step hint to verify ownership and consumer mapping

## Consumer expectations

- Crate tests can keep implementation-local fixtures, but canonical scenario truth stays under `evidence/`.
- Registry reload operations must preserve the set of asset ids unless evidence sources change.
