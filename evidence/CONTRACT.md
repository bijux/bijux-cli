# Evidence Authority Contract

## Authority
`evidence/` is the single ownership boundary for executable proof assets in this repository. Scenario assets in other roots are temporary compatibility surfaces and are governed by this contract until migrated.

## Subdomains
- `authoring`
- `battle`
- `cache`
- `compat`
- `fault`
- `operator`
- `perf`
- `compare`

## Asset classification
- Evidence asset: executable scenario, fixture, or baseline that proves runtime, policy, compatibility, failure, operator, or performance behavior.
- Schema fixture: syntax and schema-conformance fixture owned by schema policy under `configs/schema/**`.
- Test helper: non-authoritative utility input used only to make a test executable, never as product truth.

## Canonical ownership rules
- All scenario-like assets are owned by `evidence/ownership/evidence_ledger.json`.
- New scenario-like files outside approved evidence roots are forbidden.
- `examples/`, `benchmarks/`, `comparisons/`, and scenario-bearing `tests/` trees are compatibility roots under evidence governance until migration is complete.
- Crate-local fixtures are consumers of evidence authority and cannot define independent truth.

## Metadata requirements
Each governed asset must include:
- identity: `path`, `canonical_location`, `duplicate_of`
- ownership: `owner`, `evidence_class`, `consumer_surfaces`
- trust: `trust_property`, `trust_properties_protected`, `release_blocking`
- lifecycle: `decision`, `deletion_review`, `retirement_date`
- status: `implementation_status`, `why_exists`

## Enforcement
`bijux-dev-dag` is the only approved control plane for evidence governance checks:
- taxonomy and ownership reporting
- metadata completeness validation
- evidence drift and out-of-bound path rejection
