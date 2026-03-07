# Benchmark Result Format

## Purpose
Define the stable benchmark report shape, including raw measurements and metadata required for reproducibility.

## Schema authority
- `configs/schema/benchmarks/benchmark_report.schema.json`

## Required report sections
- report metadata: format version, timestamp, commit SHA, toolchain
- machine metadata: CPU, memory, host context
- scenario metadata: scenario id, benchmark class, run configuration
- raw outputs: measured values, units, sample counts
- derived outputs: summaries and ratio comparisons

## Source-of-truth rule
Published benchmark summaries must retain links to raw report files used to generate them.
