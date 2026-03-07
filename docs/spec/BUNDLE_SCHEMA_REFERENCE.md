# Bundle Schema Reference

This page indexes bundle format references and fixture examples.

## Format specs

- `docs/spec/GRAPH_BUNDLE_FORMAT_v1.md`
- `docs/spec/RUN_BUNDLE_FORMAT_v1.md`
- `docs/spec/ARTIFACT_BUNDLE_FORMAT_v1.md`
- `docs/spec/BUNDLE_MANIFEST_VERSIONING_POLICY.md`

## Fixture examples

- Minimal bundle: `evidence/compat/export_bundle/v0_1_supported/examples/minimal_bundle.json`
- Maximal bundle: `evidence/compat/export_bundle/v0_1_supported/examples/maximal_bundle.json`

## Verification surfaces

- `bijux dag import --verify-only <bundle>`
- `bijux dag fsck <bundle> --json`
