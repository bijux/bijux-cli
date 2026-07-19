---
title: Gated Command Inventory
audience: operators
type: generated-reference
status: canonical
owner: bijux-dag-docs
generated_from: bijux-dag clap help surface
---

# Gated Command Inventory

This page is generated from the live `bijux-dag` command tree. It is the
repository-owned inventory for routes that remain outside the stable
`v0.4.0` operator compatibility lane.

Stable commands belong in
[`generated-cli-reference.md`](generated-cli-reference.md). This page is
only for deliberate access to experimental, simulated, or internal routes.

## Experimental Routes

Callable by explicit path and repository-tested, but intentionally excluded from the stable public operator surface.

| Path | Lane | Availability | Opt-In |
| --- | --- | --- | --- |
| `adapters` | `experimental` | `explicit-path` | `-` |
| `adapters admit` | `experimental` | `explicit-path` | `-` |
| `adapters cache-compat` | `experimental` | `explicit-path` | `-` |
| `adapters conformance` | `experimental` | `explicit-path` | `-` |
| `adapters describe` | `experimental` | `explicit-path` | `-` |
| `adapters doctor` | `experimental` | `explicit-path` | `-` |
| `adapters dump` | `experimental` | `explicit-path` | `-` |
| `adapters ls` | `experimental` | `explicit-path` | `-` |
| `adapters reference` | `experimental` | `explicit-path` | `-` |
| `artifact fetch` | `experimental` | `explicit-path` | `-` |
| `canonical-bytes` | `experimental` | `explicit-path` | `-` |
| `canonical-diff` | `experimental` | `explicit-path` | `-` |
| `canonicalize` | `experimental` | `explicit-path` | `-` |
| `config` | `experimental` | `explicit-path` | `-` |
| `config show-effective` | `experimental` | `explicit-path` | `-` |
| `explain-plan` | `experimental` | `explicit-path` | `-` |
| `export` | `experimental` | `explicit-path` | `-` |
| `fingerprint` | `experimental` | `explicit-path` | `-` |
| `fsck` | `experimental` | `explicit-path` | `-` |
| `graph` | `experimental` | `explicit-path` | `-` |
| `graph-lint` | `experimental` | `explicit-path` | `-` |
| `hash` | `experimental` | `explicit-path` | `-` |
| `hash artifact` | `experimental` | `explicit-path` | `-` |
| `hash graph` | `experimental` | `explicit-path` | `-` |
| `hash run` | `experimental` | `explicit-path` | `-` |
| `import` | `experimental` | `explicit-path` | `-` |
| `init` | `experimental` | `explicit-path` | `-` |
| `lint` | `experimental` | `explicit-path` | `-` |
| `migrate` | `experimental` | `explicit-path` | `-` |
| `migrate dag` | `experimental` | `explicit-path` | `-` |
| `migrate inspect` | `experimental` | `explicit-path` | `-` |
| `migrate run` | `experimental` | `explicit-path` | `-` |
| `node` | `experimental` | `explicit-path` | `-` |
| `policy` | `experimental` | `explicit-path` | `-` |
| `policy show-effective` | `experimental` | `explicit-path` | `-` |
| `proof-summary` | `experimental` | `explicit-path` | `-` |
| `prove` | `experimental` | `explicit-path` | `-` |
| `run-bundle` | `experimental` | `explicit-path` | `-` |
| `show-effective-graph` | `experimental` | `explicit-path` | `-` |
| `status` | `experimental` | `explicit-path` | `-` |
| `trace-artifact` | `experimental` | `explicit-path` | `-` |
| `trace-node` | `experimental` | `explicit-path` | `-` |
| `why-cache-missed` | `experimental` | `explicit-path` | `-` |
| `why-rerun` | `experimental` | `explicit-path` | `-` |

## Simulated Routes

Modeled platform namespaces. Execution requires `BIJUX_DAG_ENABLE_SIMULATED=1` and does not claim production backends.

| Path | Lane | Availability | Opt-In |
| --- | --- | --- | --- |
| `control-plane` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane api` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane backpressure` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane cache` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane fan-in` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane idempotency` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane leadership` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane leases` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane migration` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane planning` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `control-plane sharding` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `dataset` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `dataset mapping` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `dataset staleness` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise approval` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise asset-link` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise calendar` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise credentials` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise dependency-catalog` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise export` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise incident-hook` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise queue` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise service-contract` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `enterprise webhook` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation audit-integrity` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation config-inheritance` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation delegation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation failover` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation lineage` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation policy-distribution` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation replay` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation schedule` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation sovereignty` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `federation trust-tier` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet autoscale` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet capabilities` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet drain` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet fragmentation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet gossip` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet isolation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet preemption` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet register` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet trust` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `fleet warm-pool` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance alerts` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance audit-event` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance catalog-export` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance compliance` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance contracts` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance cost` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance ownership` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance policy-check` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance promotion` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `governance tags` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident annotation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident blast-radius` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident degraded-mode` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident mode` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident readiness-review` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident repair-window` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident replay-validation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident safe-stop` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident scorecard` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `incident timeline` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability change-impact-labels` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability compatibility-fixtures` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability contract-alignment` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability medium-acceptance-gate` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability module-surface-budgets` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability production-candidate` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability public-api-review` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability release-notes-evidence` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab durability typed-contracts` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise approval` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise asset-link` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise calendar` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise credentials` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise dependency-catalog` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise export` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise incident-hook` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise queue` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise service-contract` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab enterprise webhook` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation audit-integrity` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation config-inheritance` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation delegation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation failover` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation lineage` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation policy-distribution` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation replay` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation schedule` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation sovereignty` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab federation trust-tier` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident annotation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident blast-radius` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident degraded-mode` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident mode` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident readiness-review` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident repair-window` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident replay-validation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident safe-stop` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident scorecard` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab incident timeline` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance artifact-write-profile` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance benchmark-report-governance` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance canonicalization-profile` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance large-graph-corpus` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance latency-budgets` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance memory-ceilings` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance performance-regression-gates` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance run-history-compaction` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance scheduler-churn` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab performance streaming-output` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release canary` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release checkpoint` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release classify` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release deprecation` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release evidence` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release health` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release promotion` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release rollback` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release shadow` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab release version` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security artifact-secrets` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security auth` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security authz` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security command-injection` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security data-access` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security dependency-risk` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security env-allowlist` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security filesystem-allowlist` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security malformed-input-fuzz` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security network-policy` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security override` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security override-audit` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security safe-defaults` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security secrets` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security supply-chain` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security supply-inventory` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security tenant` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `lab security trust-classes` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store amplification` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store archive` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store checksum` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store clock` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store consistency` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store index` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store journal` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store retention` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store snapshot` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |
| `state-store transaction` | `simulated` | `opt-in` | `BIJUX_DAG_ENABLE_SIMULATED` |

## Internal Routes

Maintainer-only and contract-only routes. Execution requires `BIJUX_DAG_ENABLE_INTERNAL=1`.

| Path | Lane | Availability | Opt-In |
| --- | --- | --- | --- |
| `capabilities` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability change-impact-labels` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability compatibility-fixtures` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability contract-alignment` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability medium-acceptance-gate` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability module-surface-budgets` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability production-candidate` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability public-api-review` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability release-notes-evidence` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `durability typed-contracts` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `equivalence-proof` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance artifact-write-profile` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance benchmark-report-governance` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance canonicalization-profile` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance large-graph-corpus` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance latency-budgets` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance memory-ceilings` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance performance-regression-gates` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance run-history-compaction` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance scheduler-churn` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `performance streaming-output` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release canary` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release checkpoint` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release classify` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release deprecation` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release evidence` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release health` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release promotion` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release rollback` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release shadow` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `release version` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime cancel` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime control-recovery` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime dispatch` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime events` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime execute-payload` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime heartbeat` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime intervention` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime isolation` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime pause` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime repair` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime retry` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime state` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime timeout` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime transition` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime worker-recovery` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `runtime write-discipline` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule audit` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill advance` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill cancel` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill pause` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill plan` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill resume` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill retry-failed` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill status` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule backfill summary` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule compile` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule control` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule control pause` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule control resume` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule control status` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule dedup` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule order` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule preview` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule queue` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule queue dispatch` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule queue status` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule queue update` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule sla` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule submit` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule throttle` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `schedule validate` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security artifact-secrets` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security auth` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security authz` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security command-injection` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security data-access` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security dependency-risk` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security env-allowlist` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security filesystem-allowlist` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security malformed-input-fuzz` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security network-policy` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security override` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security override-audit` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security safe-defaults` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security secrets` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security supply-chain` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security supply-inventory` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security tenant` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `security trust-classes` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `semantic-portability` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
| `version-inspect` | `internal` | `opt-in` | `BIJUX_DAG_ENABLE_INTERNAL` |
