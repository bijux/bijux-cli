# Dev CLI Release Truth Policy

`dev cli release *` is the canonical source of release truth.

Rules:
- Release status and readiness claims must come from `bijux dev cli release status` and `bijux dev cli release readiness`.
- Release evidence claims must come from `bijux dev cli release evidence`.
- Release blockers and unresolved gaps must come from `bijux dev cli release gaps`.
- CI must publish `artifacts/status/dev_cli_release_truth_bundle.json` and enforce freshness/blocker thresholds.
