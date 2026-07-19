# Changelog

All notable changes to **bijux-dag-app** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-19

### Added
- First public crates.io release of `bijux-dag-app` as the application
  orchestration layer for `bijux-dag`.
- Stable and prelude import lanes for supported command embedding and typed
  application responses.
- Command orchestration for graph validation, planning, execution, replay,
  inspection, comparison, verification, and cache operations.
- Typed response models and machine-readable output contracts before terminal
  rendering.
- Configuration and runtime-input resolution with explicit source and
  precedence reporting.
- Run and node inspection views that expose retained status, failures,
  attempts, logs, artifacts, branch decisions, and trigger outcomes.
- Replay planning, focused repair, run comparison, and strict post-run
  verification flows.
- Artifact inspection, export, import, promotion, and integrity orchestration
  over the artifacts and runtime crates.
- Stable, experimental, simulated, and internal route classification with
  explicit discovery and opt-in boundaries.
- Thin orchestration boundaries that leave graph truth in
  `bijux-dag-core`, execution in `bijux-dag-runtime`, and retained formats in
  `bijux-dag-artifacts`.
