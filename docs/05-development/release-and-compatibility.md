# Release And Compatibility

## Goal

Ship from verified state, keep compatibility reasoning explicit, and make
rollback practical when a release is wrong.

```mermaid
flowchart TD
    A[Candidate commit] --> B[CI passes]
    B --> C[Maintainer checks pass]
    C --> D[Compatibility review passes]
    D --> E[Tag release]
    E --> F[Publish]
```

```mermaid
sequenceDiagram
    participant C as Candidate
    participant T as Tests and checks
    participant R as Release workflow
    participant P as Published version
    C->>T: verify behavior
    T->>R: green evidence
    R->>P: publish tagged version
    P-->>R: rollback to last good version if needed
```

## Release Rules

- release from a clean, verified commit
- prefer the repository workflows over manual publish sequences
- check runtime identity and compatibility before tagging
- keep rollback tied to released versions, not local artifacts
- keep workspace manifests on the active dev line; let tagged publish workflows
  stamp the exact release version into their temporary release tree

## Common Review Commands

```bash
cargo test --locked --workspace
python3 -m pytest crates/bijux-cli-python/tests/python/test_runtime_parity.py
BIJUX_ENABLE_STABLE_PYPI_PARITY=1 python3 -m pytest -m nightly crates/bijux-cli-python/tests/python/test_stable_release_compatibility.py
bijux dev cli status --format json --no-pretty
bijux dev cli parity --format json --no-pretty
make docs-check
```

## Compatibility Standard

The Rust runtime owns current behavior. Compatibility review still matters in
two directions:

- current `bijux-cli` vs current `bijux-cli-python`
- current `bijux-cli-python` vs the repository's configured stable PyPI
  compatibility baseline, currently `bijux-cli==0.2.0`

## Runtime Migration Baseline

The Python-core migration is no longer an active documentation track. The
current state is:

- the `bijux` command surface is owned by the Rust runtime
- the Python package remains a compatibility surface, not a second independent
  runtime
- compatibility review is preserved through the two live comparisons above
- cutover decisions and rollback decisions are tied to published versions, not
  to checked-in snapshots or migration-era capture files

## Release Decision Rules

- release as a minor version only when documented behavior remains compatible
- release as a major version when documented behavior changes incompatibly
- keep `python -m pip install bijux-cli` and tagged publish workflows aligned
  with the same runtime identity
- if a release regresses compatibility, roll back to the last known-good
  published version in the affected channel

## Honest Limit

A green release checklist is not a guarantee that no regression exists. It is a
claim that the current evidence supports shipping more than delaying.

## Where To Go Deeper

- [Quality and change management](../04-architecture/quality-and-change-management.md)
- [Runtime and distribution](../04-architecture/runtime-and-distribution.md)
- [Integrations and routed runtimes](../06-reference/integrations-and-routed-runtimes.md)
