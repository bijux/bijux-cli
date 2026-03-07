# Bijux CLI Integration Contract

## Purpose
Define boundaries between root `bijux` command surfaces and `bijux dag` semantics.

## Command ownership
- `bijux dag` owns DAG semantics and runtime truth surfaces.
- root `bijux` may compose orchestration UX but must not alter DAG identity/replay semantics.

## Integration rule
All composed CLI surfaces must preserve `bijux-dag` JSON contracts and exit semantics.
