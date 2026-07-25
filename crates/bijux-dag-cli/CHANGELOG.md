# Changelog

All notable changes to **bijux-dag-cli** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.1 – 2026-07-25

### Changed
- Advanced `bijux-dag-cli` and its application dependency constraint to the
  `v0.4.1` release line.
- Canonicalized README links to executable examples, operator workflows,
  package contracts, and deployed documentation.

### Fixed
- Added a dedicated `bijux-dag-cli` GHCR target matching its crates.io package
  identity and installed `bijux-dag` executable.

## 0.4.0 – 2026-07-24

### Added
- First public crates.io release of `bijux-dag-cli`, installing the
  `bijux-dag` executable.
- Visible stable command surface for validation, planning, execution, replay,
  run inspection, artifact work, cache operations, and verification.
- `commands` inventory for distinguishing stable, experimental, simulated,
  and internal route lanes.
- Shell completion generation for Bash, Zsh, Fish, Elvish, and PowerShell.
- Human-readable and machine-readable command output supplied by the
  application layer.
- Explicit process exit mapping for successful commands, usage errors,
  application failures, and unexpected panics.
- Panic containment at the executable boundary so internal failures do not
  escape without a deterministic nonzero exit.
- Environment-gated access to simulated and maintainer-only command
  namespaces.
- Stable container and branch workflow entrypoints without embedding runtime
  behavior in the binary crate.
- Thin startup delegation to `bijux-dag-app`, preserving a narrow executable
  ownership boundary.
