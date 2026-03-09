# Newly Ported Command Parity

Date: 2026-03-09

This report tracks commands ported in the 201–220 batch.

| Command | Python Capture Available | Output Parity Diff | Exit-Code Parity Diff | Stream Parity Diff | Baseline Status |
|---|---|---|---|---|---|
| `status` | yes (`bijux_status_text`) | different | match | stderr match, stdout diff | still-partial |
| `audit` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `docs` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `sleep 0` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `cli config get` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `cli config set` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `cli self-test` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `cli plugins list` | no direct capture (`plugins list` root only) | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `cli plugins inspect` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `dev cli routes` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `dev cli registry` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `dev cli env` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `dev cli doctor` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |
| `dev cli contracts` | no | unavailable | unavailable | unavailable | baseline-complete (rust-only) |

## Notes

1. For commands without Python captures in `artifacts/current-python-behavior-lock.json`, parity is marked unavailable rather than inferred.
2. Baseline-complete for rust-only commands means the command is fully routed and tested at binary/core levels in this repository.
