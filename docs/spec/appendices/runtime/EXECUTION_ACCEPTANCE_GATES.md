# Execution acceptance gates

Required acceptance checks:
- Node and run state-machine legality checks.
- Deterministic scheduling under `jobs=1` and `jobs>1`.
- Stable ready-node tie-break ordering.
- Deterministic failure propagation behavior.
- Retry backoff metadata persisted in node traces.
- Timeout failures distinguishable from execution failures.
- Cancellation writes complete final manifest.
- Selection/exclusion runs emit manifest and trace completeness.
- Replay behavior does not depend on ambient state.
- Latest symlink updates preserve historical run integrity.
- Run ID collision handling is deterministic.
- `clean_env` and `deny_env` interactions are deterministic.
- `deny_network` behavior is consistent across adapter classes.
- Output validation covers missing, extra, duplicate, malformed outputs.
- Manifest node totals must equal per-node trace totals.
