# Geo-ready acceptance gates

A deployment is geo-ready only when all gates pass:

- registry readiness
- scheduler readiness
- lineage readiness
- observability readiness

## Required evidence

- regional ownership and write-routing conformance report
- split-brain simulation report
- replication lag simulation report
- region-loss failover simulation report
- cross-region lineage queryability report
- observability regional aggregation validation

## Failure behavior

If any gate fails, geo federation remains in non-production status and cross-region failover automation must stay disabled.
