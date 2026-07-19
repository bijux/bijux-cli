# bijux-dev Package Contract

## Responsibility

`bijux-dev` is the private repository control plane for policy validation,
suite orchestration, diagnostic reporting, governed evidence, and release
verification.

## Binary Authorities

- `bijux-dev-cli` owns repository and product diagnostic views, maintenance
  audits, documentation publishing commands, and structured maintainer reports.
- `bijux-dev-dag` owns governed suite catalogs, policy and contract execution,
  DAG evidence verification, and release-proof composition.
- `contracts/foundation/maintainer_command_surface.v1.json` governs the visible
  `bijux-dev-dag` root command surface.

Commands with similar names must delegate to one owner or remain intentionally
different with a documented boundary.

## Dependency Direction

- The package may read public product contracts and depend on product crates for
  verification.
- Product crates must not depend on `bijux-dev`.
- Maintainer reports must not become alternate implementations of CLI or DAG
  behavior.
- Repository automation may invoke package commands but must not duplicate
  their policy logic.

## Execution Contract

- Validation commands return non-zero when required selected checks fail.
- Advisory, slow, internal, and narrowed selections remain explicit in command
  evidence.
- Commands identify governed outputs and direct transient output to
  `artifacts/`.
- Generated evidence records enough source and producer identity to be reviewed
  against its contract.
- Broad suites preserve component failures and aggregate final status rather
  than hiding failure through early success or partial reporting.

## Make And CI Boundary

Make targets and GitHub workflows are entrypoint adapters. They own environment
setup and orchestration but delegate repository policy and suite behavior to
the package or another named authority. A wrapper must preserve command status,
selection, and final evidence.

## Exclusions

The package does not own:

- end-user CLI semantics;
- DAG graph, runtime, backend, or artifact semantics;
- Python bridge behavior;
- organization-wide standards synchronized from `bijux-std`.

Changing a product contract requires the owning product implementation and
tests; changing a synchronized standard requires the upstream standards
authority.
