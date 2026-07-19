# Repository Work Routing

This governed evidence maps repository-level work classes to their owning crate
and expected evidence location. The executable authority is
`contracts/foundation/repository_work_routing.v1.json`; the contract test
rejects unknown classes, owner drift, missing evidence roots, and incomplete
coverage in this table.

## Governed Responsibilities

| Work class | Owning crate | Evidence location | Responsibility |
| --- | --- | --- | --- |
| `foundation-ownership-boundary` | `bijux-dev` | `contracts/foundation/` | shared package ownership, dependency direction, and repository boundary contracts |
| `foundation-governance` | `bijux-dev` | `docs/reports/governance/` | reviewable evidence for repository policy, authority, and drift decisions |
| `foundation-compatibility-lanes` | `bijux-cli` | `contracts/schemas/` | machine-readable compatibility and command-envelope boundaries |
| `foundation-release-gate` | `bijux-dev` | `crates/bijux-dev/tests/` | executable acceptance criteria for release and publication |
| `foundation-operator-diagnostics` | `bijux-cli` | `crates/bijux-cli/src/interface/cli/handlers/` | operator-facing diagnosis, evidence, and remediation behavior |

## Routing Decision

Classify work by the durable surface it changes:

- use ownership boundary when a shared contract changes who owns or may depend
  on a surface;
- use governance when the evidence explains repository policy or observed
  drift without defining product behavior;
- use compatibility lanes when serialized or command-facing consumer meaning
  changes;
- use release gate when publication acceptance or its executable proof changes;
- use operator diagnostics when users receive different diagnosis or
  remediation behavior.

If none of these classes fits, change the contract deliberately before work is
accepted. Do not use an uncategorized class or encode delivery order in a
lasting identifier.

## Verification

Run:

```sh
cargo nextest run -p bijux-dev \
  --test foundation_repository_work_routing_contracts
```

Passing this suite proves that the routing contract and this retained evidence
agree. It does not prove that a particular implementation is correct; the
owning crate's behavioral tests remain responsible for that claim.
