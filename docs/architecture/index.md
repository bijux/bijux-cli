# Architecture

## Purpose
This section collects architectural documentation that explains how bijux-cli is structured and why those structures exist. It serves as a bridge between conceptual guarantees and practical execution.

## Scope
It contains a walk-through of the execution path and the explicit decision rules used to preserve invariants. It does not describe implementation details line by line.

## Audience
Engineers who need to understand the architectural choices behind bijux-cli, such as contributors and integrators, should start here.

## Index
- [Decision rules](decision-rules.md)
- [Walk-through](walkthrough.md)
- [ADR: CLI binary ownership](adr-cli-binary-ownership.md)
- [Config crate ownership](config-crate-ownership.md)
- [Config domain invariants](config-domain-invariants.md)
- [Config key/value parity coverage](config-key-value-parity.md)
- [Config file and path behavior](config-file-path-behavior.md)
- [Config root parity report](config-root-parity-report.md)
- [Config get parity report](config-get-parity-report.md)
- [Config get post-parity improvements](config-get-post-parity-improvements.md)
- [Config set parity report](config-set-parity-report.md)
- [Config set post-parity improvements](config-set-post-parity-improvements.md)
- [Config unset/clear/reload parity report](config-unset-clear-reload-parity-report.md)
- [Config export/load parity report](config-export-load-parity-report.md)
- [Python config UX ambiguities review](python-config-ux-ambiguities-review.md)
- [Config post-parity candidates](config-post-parity-candidates.md)
- [Config parity matrix](config-parity-matrix.md)
- [Rust config baseline parity milestone](rust-config-baseline-parity-milestone.md)
- [Crate boundaries](crate_boundaries.md)
- [Runtime identity law](runtime_identity_law.md)
- [Test policy](test-policy.md)
- [Test review checklist](test-review-checklist.md)
- [Test quality priorities](test-quality-priorities.md)
- [Python package convergence report](python-package-convergence-report.md)
- [Python package baseline](python-package-baseline.md)
- [Python public API lifecycle](python-public-api-lifecycle.md)
