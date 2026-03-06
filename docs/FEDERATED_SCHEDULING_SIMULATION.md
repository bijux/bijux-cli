# Federated scheduling simulation harness

## Scenario coverage

Required scenarios:
- overflow burst routing between domains
- failover to alternate domain after downstream impairment
- policy conflict resolution during delegation

## Validation checks

- delegation determinism under repeated identical inputs
- flow-control enforcement under burst pressure
- delegated-run lineage continuity
- cross-domain audit exchange completeness

## Failure classes

- transient child-domain execution impairment
- persistent trust incompatibility
- cross-domain policy mismatch

Each class must map to an explicit delegation failure action.
