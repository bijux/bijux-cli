# Evidence Asset Authoring Rules

## Naming convention
- File names use lowercase kebab-case with stable intent nouns.
- Scenario metadata ids use `^[a-z0-9][a-z0-9._-]*$` and never embed dates.
- Positive fixtures avoid `good` in names; use behavior intent names.
- Negative fixtures start with `invalid-` or `reject-`.

## Directory depth policy
- Class root depth: `evidence/<class>/...`
- Scenario assets: max depth 4 from `evidence/<class>`.
- Metadata registries and generated maps live only under `evidence/_meta/`.

## Scenario id and stable references
- Every executable scenario has a stable id in metadata.
- Consumers reference scenarios by id and canonical path, not ad-hoc relative paths.
- Renames require registry update preserving prior id aliases if consumers still depend on them.

## Classification boundaries
- `battle`: release-trust executable proof.
- `authoring`: examples/patterns for DAG authoring behavior.
- `perf`: workload and budget checks.
- `compare`: cross-system scenario comparisons.
- `cache`, `compat`, `fault`, `operator`: domain-specific semantics only.

## Normative versus derived
- Normative assets define expected system truth and gate behavior.
- Derived assets are generated from normative assets and must be reproducible.
- Derived assets must not be edited manually.

## Hand-authored versus generated
- Hand-authored assets are maintained directly under evidence class domains.
- Generated assets must include generation source in metadata and reside under `evidence/_meta/maps` or `evidence/reports`.
