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

## Historical Capture Notes

Older side-by-side appendix pages were reduced into this summary because they
duplicated generated capture artifacts and had become harder to maintain than
the underlying evidence.

The archived Python-vs-Rust captures showed these durable conclusions:

- stdout payloads still differed for `status`, `doctor`, `plugins list`, `config`,
  `history`, and `dev --help`
- stderr parity was mostly clean, but plugin failure cases and some config error
  paths still differed
- sample timing captures existed for Rust, but they were not a complete
  cross-runtime benchmark and should not be treated as current performance truth
