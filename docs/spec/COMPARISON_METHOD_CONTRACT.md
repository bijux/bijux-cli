# Comparison Method Contract

## Purpose
Define method rules for comparing benchmark outputs and publishing regressions.

## Comparison inputs
- Current benchmark report
- Baseline benchmark report
- Maximum allowed regression ratio threshold

## Method rules
- Compare only matching scenario IDs and benchmark classes.
- Missing baseline rows are reported as `unscored` and cannot justify performance claims.
- Ratio comparisons must use the same unit family (`ms`, `us`, bytes, throughput values).
- Threshold interpretation must be explicit: pass, warn, or fail.

## Command surface
Primary command: `cargo run -p bijux-dev-dag -- benchmark-compare --current <path> --baseline <path> --max-regression-ratio <value>`

## Output requirements
Comparison output must include:
- scenario ID
- benchmark class
- baseline value
- current value
- ratio
- threshold
- status
