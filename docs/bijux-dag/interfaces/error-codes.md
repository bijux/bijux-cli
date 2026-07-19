---
title: Error Codes
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Error Codes

Use this page when automation, release review, or operator triage needs the
stable public `bijux-dag` error identifiers rather than free-form prose.

For the deeper reference wording, open
[Error Codes Reference](error-codes.md). Public error code behavior
is governed by [Error Contract](../../spec/ERROR_CONTRACT.md).

## Public Registry

| Code | Category | Owner | Meaning |
| --- | --- | --- | --- |
| `BJX-PARSE-001` | `parse` | `bijux-dag-core` | Input is not valid DAG JSON |
| `BJX-SCHEMA-001` | `schema` | `bijux-dag-core` | JSON shape violates schema contract |
| `BJX-VALIDATION-001` | `validation` | `bijux-dag-core` | Semantic graph validation failed |
| `BJX-CONFIG-001` | `config` | `bijux-dag-app` | Invalid configuration input |
| `BJX-POLICY-001` | `policy` | `bijux-dag-runtime` | Policy denied requested behavior |
| `BJX-EXEC-001` | `execution` | `bijux-dag-runtime` | Node execution failed |
| `BJX-IO-001` | `io` | `bijux-dag-app` | Filesystem or artifact I/O operation failed |
| `BJX-REPLAY-001` | `replay` | `bijux-dag-app` | Replay mismatch against recorded artifacts |
| `BJX-CACHE-001` | `cache` | `bijux-dag-runtime` | Cache contract or proof mismatch |
| `BJX-COMPAT-001` | `compatibility` | `bijux-dev-dag` | Compatibility contract violation |
| `BJX-INTERNAL-001` | `internal` | `bijux-dag-cli` | Unexpected internal failure path |

## Usage rules

- stable automation should branch on the public code, not on human prose
- category names stay stable so error lanes remain queryable
- owner crates identify who must review semantic changes to a public code
- internal implementation details may evolve without changing the public code
  meaning
