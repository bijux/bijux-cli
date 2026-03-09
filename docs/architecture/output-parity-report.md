# Output Parity Report

Date: 2026-03-09
Compared commands with Python captures:
- `status` (`bijux_status_json_no_pretty`)
- `doctor` (`bijux_doctor`)
- `plugins list` (`bijux_plugins_list`)

## Current result summary
- Structural parity confirmed for command success envelope presence and `status` marker.
- Exact payload parity is not yet complete for all compared commands.
- `doctor` payload structure differs (Python includes deeper product summary fields; Rust currently emits focused install checks).
- `plugins list` includes directory metadata in Rust response.

## Action status
- Captured as known parity gaps pending command-depth convergence.
- Covered commands remain under parity regression tests for exit-code and stream behavior.
