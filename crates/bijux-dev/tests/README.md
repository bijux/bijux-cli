# Repository Governance Tests

`bijux-dev` owns cross-crate checks that cannot be expressed honestly inside a
single product crate. These tests verify repository architecture, evidence,
documentation, release policy, generated references, and Make entrypoints.
They are not a second implementation test suite.

## Boundary Tests

- `no_core_io.rs`, `no_runtime_in_core.rs`, and `no_cli_in_runtime.rs` enforce
  dependency direction.
- `dependency_boundary_contracts.rs`, `crate_taxonomy_guardrails.rs`, and
  `source_layout_guardrails.rs` protect package and source ownership.
- `root_tools_forbidden_contracts.rs` and `file_size_guardrails.rs` prevent
  repository structure from degrading silently.

## Evidence And Documentation

Evidence tests connect registries, consumers, generated reports, and freeze
rules. Documentation tests validate source references, product wording,
release boundaries, examples, reproducibility, security claims, and known
limitations. A failing docs contract should be resolved by aligning the
authoritative source and its readers, not by deleting the assertion.

## Test-Lane Governance

The fast-suite and zero-coverage contracts verify that critical runtime,
artifact, planner, scheduler, and adapter tests remain assigned to the intended
lane. Frozen-gate tests protect the background full-suite command and its
summary artifacts.

## Focused Runs

```bash
cargo nextest run -p bijux-dev --test dependency_boundary_contracts
cargo nextest run -p bijux-dev --test docs_source_reference_contracts
cargo nextest run -p bijux-dev --test evidence_governance_contract
cargo nextest run -p bijux-dev --test release_validation_suite_contracts
```

Start with the named failing binary. If a generated report is stale, run its
owning maintainer command and review the diff; do not hand-edit generated
evidence. Run repository-wide gates only after the focused contract passes.
