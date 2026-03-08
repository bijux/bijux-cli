# Dev DAG Legacy Command Namespace Report

Legacy handling policy:

- legacy-only commands must stay outside primary release-critical command packs
- legacy commands remain available for compatibility but excluded from primary help narratives
- advisory/legacy separation is enforced via command pack suites and contracts

Primary governance anchors:

- `configs/suites/dev_dag_release_critical_pack.json`
- `configs/suites/dev_dag_maintenance_pack.json`
- `crates/bijux-dev-dag/tests/dev_dag_contraction_461_480_contracts.rs`
