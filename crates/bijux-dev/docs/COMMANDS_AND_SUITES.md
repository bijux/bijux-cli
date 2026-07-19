# Commands And Suites

Maintainer commands turn named repository intentions into reproducible
selection and status. Suites centralize what runs so Make and CI wrappers do
not carry independent policy.

## Command Authorities

The visible `bijux-dev-dag` root order is governed by
`contracts/foundation/maintainer_command_surface.v1.json`. Command families
cover control, verification, release, contracts, evidence, docs, performance,
reporting, repository operations, and generators.

`bijux-dev-cli` owns report-oriented repository and product diagnostics. Its
entrypoint returns stdout, stderr, and exit status as an `AppRunResult` for
testability.

## Suite Metadata

Each suite has a durable identifier, group, domain, effect, and selection
policy. Selection records:

- requested group or domain;
- selected suite identifiers;
- slow and internal inclusion;
- disabled entries and override source;
- advisory or required mode;
- fail-fast policy.

An unselected suite is not passing. A disabled suite remains visible with its
reason.

## Aggregate Execution

Non-fail-fast aggregates run every selected component, retain every failure,
and return nonzero when any required component fails. Fail-fast aggregates
stop only when their declared contract permits it.

Wrappers must preserve status through logging and pipes. Starting a process or
printing an artifact path is not completion evidence.

## Advisory Mode

Advisory execution records findings without blocking status where policy
allows. Its output must be labeled advisory and cannot support a claim that a
required gate passed.

Narrowed filters, excluded slow work, simulated checks, and partial package
runs are similarly explicit in evidence.

## Overrides

Suite overrides come from named, validated configuration. Unknown suite IDs,
malformed entries, and contradictory selection fail. Overrides cannot silently
remove a required release check.

## Adding A Suite

1. Choose a durable domain identifier.
2. Define effect and required/advisory behavior.
3. Add selection and dispatch implementation.
4. Ensure result and component evidence are retained.
5. Add catalog, filter, status, and command-surface tests.
6. Delegate Make/CI wrappers to the new authority.

Do not encode sequence numbers, sprint language, or one-off delivery names in
suite IDs.

## Verification

Suite catalog, dispatch, control-plane, command-surface, release-validation,
Make entrypoint, frozen-gate, and test-lane contracts prove selection and
status behavior.
