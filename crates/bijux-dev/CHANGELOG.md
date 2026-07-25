# Changelog

All notable changes to **bijux-dev** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.1 – 2026-07-25

### Added

- Added release contract coverage requiring exact parity between public
  crates.io packages and GHCR package slugs.
- Added README public-link and Mermaid source checks to documentation
  verification.
- Added the existing reusable documentation deployment workflow to the version
  tag release coordinator.

### Changed

- Advanced the maintainer control plane and its workspace dependency
  constraints to the `v0.4.1` release line.
- Documented trusted PyPI publication and canonicalized maintainer README links
  to repository contracts and deployed handbook pages.

The package remains repository-internal and is not a registry release target.

## 0.4.0 – 2026-07-24

### Added

- Established `bijux-dev` as the private, non-published maintainer control
  plane for repository policy, diagnostics, governed evidence, and release
  verification.
- Added the `bijux-dev-cli` binary for repository health, runtime and package
  diagnostics, documentation publishing, maintenance audits, and structured
  maintainer reports.
- Added the `bijux-dev-dag` binary for suite discovery and execution, contract
  governance, DAG evidence verification, comparison reporting, and release
  proof.
- Governed the visible `bijux-dev-dag` root command inventory through
  `contracts/foundation/maintainer_command_surface.v1.json`.
- Added reusable suites for repository layout, documentation, policy,
  contracts, tests, and release readiness with explicit selection and
  aggregate status.
- Added evidence producers and validators that record source identity,
  producer identity, output location, and integrity information for checked-in
  specifications and reports.
- Added structured maintainer views for release state, repository health,
  runtime surfaces, Python compatibility, Rust documentation, configuration,
  and evidence access.

### Changed

- Made product ownership explicit: maintainer tooling may inspect public CLI
  and DAG contracts, but it does not redefine routing, graph semantics,
  scheduling, backend execution, artifact meaning, or Python bridge behavior.
- Kept product dependency direction one-way by allowing `bijux-dev` to depend
  on product crates while prohibiting product crates from depending on
  maintainer tooling.
- Required broad suite execution and Make or CI wrappers to preserve component
  failures, selected scope, final status, and retained evidence instead of
  treating process start or partial output as success.
- Aligned release verification with clean-tree formatting, linting, tests,
  documentation, package inspection, publish dry-runs, and CLI smoke evidence
  for the public `bijux-cli` and DAG crate family.
- Directed transient diagnostics, generated run products, and local reports to
  `artifacts/`, while reserving `docs/spec`, `docs/reports`, and other governed
  paths for outputs with named producers and contract coverage.

The package remains repository-internal at this version. This entry records its
workspace contract; it does not claim that `bijux-dev` was published to a
registry.
