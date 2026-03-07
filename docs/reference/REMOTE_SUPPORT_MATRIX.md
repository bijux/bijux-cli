# Remote Support Matrix

| Surface | Status | Evidence source |
| --- | --- | --- |
| Task lease and heartbeat semantics | contract-implemented | `remote_worker_protocol_contracts` |
| Duplicate dispatch prevention | contract-implemented | `remote_worker_protocol_contracts` |
| Transport integrity and event ordering | contract-implemented | `remote_worker_protocol_contracts` |
| Capability report (`dag capabilities --backend remote`) | implemented | CLI/app contract tests |
| Remote execution backend | simulated | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |

## Support rule

This matrix is authoritative only when linked contract suites are green.
