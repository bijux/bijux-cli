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
- Schema fixture: syntax and schema-conformance fixture owned by schema policy under `configs/dag/schema/**`.
- Test helper: non-authoritative utility input used only to make a test executable, never as product truth.

## Canonical ownership rules
- All scenario-like assets are owned by `evidence/ownership/evidence_ledger.json`.
- New scenario-like files outside approved evidence roots are forbidden.
- `examples/`, `benchmarks/`, and `comparisons/` are retired proof roots and must not be reintroduced.
- Scenario-bearing `tests/` trees are constrained compatibility surfaces and cannot become proof-asset authorities.
- Crate-local fixtures are consumers of evidence authority and cannot define independent truth.

## Family boundaries
- `battle` proves end-to-end trust properties and consumes family evidence; it does not own cache/compat/fault canonical fixtures.
- `cache` owns cache-integrity and replay-fixture assets under `evidence/cache/**`.
- `compat` owns supported-versus-unsupported compatibility decisions for graph schema, run directory, and export bundles.
- `fault` owns explicit fault classes and expected system reactions for resilience behavior.
- `authoring` negatives are validation-authoring contracts and must not be used as runtime fault-class proof.

## Shared usage policy
- Shared usage is allowed only when one canonical asset is consumed by multiple suites without changing semantics.
- Duplicate assets are preferred when consumer semantics diverge or naming/diagnostics must stay domain-specific.
- Shared usage must be declared in metadata and verified by drift checks.

## Metadata requirements
Each governed asset must include:
- identity: `path`, `canonical_location`, `duplicate_of`
- ownership: `owner`, `evidence_class`, `consumer_surfaces`
- trust: `trust_property`, `trust_properties_protected`, `release_blocking`
- lifecycle: `decision`, `deletion_review`, `retirement_date`
- status: `implementation_status`, `why_exists`

## Related schemas

- `configs/dag/schema/evidence_asset.schema.json`
- `configs/dag/schema/evidence_authoring_metadata.schema.json`
- `configs/dag/schema/evidence_battle_metadata.schema.json`
- `configs/dag/schema/evidence_cache_metadata.schema.json`
- `configs/dag/schema/evidence_compare_metadata.schema.json`
- `configs/dag/schema/evidence_compat_metadata.schema.json`
- `configs/dag/schema/evidence_family.schema.json`
- `configs/dag/schema/evidence_fault_metadata.schema.json`
- `configs/dag/schema/evidence_perf_metadata.schema.json`

## Enforcement
`bijux-dev-dag` is the only approved control plane for evidence governance checks:
- taxonomy and ownership reporting
- metadata completeness validation
- evidence drift and out-of-bound path rejection
