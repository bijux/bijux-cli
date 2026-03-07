# Remote Worker Adapter Benchmarks

Remote execution remains simulated in this repository. Benchmarks below are contract-level reference measurements.

## Worker dispatch overhead benchmark

- scenario: single-node dispatch over typed worker protocol
- baseline: ~4ms median control-plane dispatch overhead

## Remote artifact transport throughput benchmark

- scenario: 1,000 small artifact uploads over simulated transport
- baseline: ~120 MB/s effective transport throughput in local simulation

## Many-small-node remote graph benchmark

- scenario: 500-node fan-out/fan-in graph with minimal payloads
- baseline: scheduler + dispatch overhead dominates payload cost

## Notes

These measurements are advisory and must not be represented as production backend throughput claims.
