# Evidence Authority Contract

## Purpose
This directory is the repository authority for executable evidence sources that validate behavior, performance, compatibility, and operator trust.

## Scope
The governed evidence roots are:
- `benchmarks/`
- `comparisons/`
- `examples/`
- `tests/`
- `crates/*/tests/fixtures/`

## Canonical Taxonomy
- `authoring`: authoring and schema correctness
- `battle`: core trust workflows and failure semantics
- `compat`: version and format compatibility
- `fault`: adversarial and resilience behavior
- `perf`: measurable throughput, latency, and resource envelopes
- `compare`: cross-system capability and behavior comparison
- `operator`: inspection, diagnostics, and usability for operations

## Governance Rules
- Every governed evidence file must be represented in `evidence/ownership/evidence_ledger.json` unless explicitly exempt.
- Every ledger entry must define owner, evidence class, trust property, and lifecycle decision.
- New files under governed roots are rejected unless they are added to the ledger in the same change.
- Duplicated evidence concepts are merged to one canonical source, and redundant files are removed.

## Lifecycle Decisions
- `keep`: canonical and retained in current root
- `merge`: concept overlaps another source and must be consolidated
- `move`: retained but migrated to future unified evidence root
- `delete`: retired because duplicated, shallow, or unowned

## Enforcement
- `crates/bijux-dev-dag/tests/evidence_governance_contract.rs` validates:
  - ownership metadata completeness
  - decision taxonomy validity
  - governed root coverage
  - freeze policy for unmanaged additions

## Deliverables Covered by This Contract
- Evidence authority proposal and taxonomy
- Full inventories for benchmarks, comparisons, examples, tests, and crate fixtures
- Ownership ledger with explicit decisions for examples, benchmark scenarios, comparison scenarios, and test fixture families
