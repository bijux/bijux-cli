# Evidence Taxonomy and Grade Definitions

## Evidence Classes
- `authoring`
- `battle`
- `compat`
- `fault`
- `perf`
- `compare`
- `operator`

## Grade Requirements
### Battle-grade
- Protects at least one named trust property in runtime semantics.
- Exercises multi-step behavior, not single-function parsing.
- Verifies outcomes with deterministic assertions.

### Perf-grade
- Includes a stable scenario identity and measurable metric target.
- Has an owned baseline or threshold contract.
- Can be executed repeatedly with comparable conditions.

### Comparison-grade
- Has a bijux-executable scenario counterpart.
- States the capability axis being compared.
- Produces actionable deltas, not narrative-only notes.

### Authoring/example-grade
- Demonstrates one canonical authoring shape.
- Is parseable and validation-safe under current schema.
- Has clear ownership and version metadata.

## Trust Property Fields
Each ledger entry must include at least one primary trust property, such as:
- `determinism`
- `failure-propagation`
- `cache-correctness`
- `replay-equivalence`
- `artifact-integrity`
- `operator-inspection`
- `schema-compatibility`
- `resource-accounting`
