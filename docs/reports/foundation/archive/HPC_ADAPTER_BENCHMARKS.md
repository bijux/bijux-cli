# HPC Adapter Benchmark Report

Status: contract benchmark baseline for simulated HPC adapter surfaces.

## Queue-submit overhead benchmark

- Scenario: `hpc-queue-submit-overhead`
- Focus: submission path overhead and scheduling metadata serialization.

## Large staged dataset benchmark

- Scenario: `hpc-large-staged-dataset`
- Focus: staged input/output path and artifact collection overhead.

## Long-latency polling benchmark

- Scenario: `hpc-long-latency-polling`
- Focus: polling strategy behavior under delayed scheduler state propagation.

## Local vs HPC overhead benchmark

- Scenario: `local-vs-hpc-overhead`
- Focus: normalized overhead comparison on equivalent graph shapes.
