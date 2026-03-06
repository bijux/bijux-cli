# API resource contract draft

This document defines the minimal service control-plane resource model for the future `dag-api`.

## Resources

- DAGs
- DAG versions
- runs
- artifacts
- schedules
- policies

## Control-plane operations

- submit
- cancel
- pause
- resume
- retry
- replay
- export
- verify

## Storage and reproducibility

- DAG registry storage is abstracted for filesystem and database implementations.
- Policy bundles are versioned to make decisions reproducible.
- Schedule definitions are separated from execution submissions.

## CLI compatibility mapping

Current `bijux-dev-dag` commands map to future service operations as follows:

- `checks run` -> repository validation endpoint
- `contracts run` -> contract execution endpoint
- `schedule validate` -> schedule compile endpoint
- `schedule preview` -> schedule simulation endpoint
- `observability-report` -> run observability report endpoint

The mapping keeps command semantics stable when CLI becomes a thin service client.
