# Run Directory Filesystem Guarantees

## Purpose

Define required filesystem and run-directory behavior for durable run records.

## Required guarantees

- deterministic run directory layout and file naming
- deterministic artifact path generation
- deterministic metadata ordering for machine-readable files
- concurrency-safe run directory creation
- recovery behavior after crashes and partial writes
- repair behavior for partial and corrupted run directories
- migration compatibility for supported run-dir schema versions
- portability behavior across filesystem path conventions

## Integrity checks

- corrupted event log detection
- corrupted node metadata detection
- missing metadata recovery handling
- consistency verification for manifest, node traces, and output indices

## Safety checks

- filesystem permission handling
- filesystem race condition resistance
- atomic write guarantees for critical metadata files
- corruption stress and recovery benchmarking coverage
