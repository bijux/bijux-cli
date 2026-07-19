# Fixtures And Builders

Fixtures provide stable inputs for graph, runtime, artifact, app, and CLI
contracts. Builders make semantic intent visible and reduce irrelevant JSON
noise.

## Graph Builders

Canonical builders cover chain, diamond, fan-out, disconnected, retry,
timeout, cache, replay, branch, and failure shapes. Workflow builders add
map/reduce, semantic map/reduce, and branch/join examples.

Builders use `bijux-dag-core` types and `SPEC_VERSION`. Defaults must be valid
under current product contracts. A newly required field is added explicitly;
the testkit must not conceal it with a stale compatibility default.

## Repository Fixtures

Loaders resolve paths relative to a supplied crate manifest directory and the
workspace root. Text, JSON, and typed loaders report the requested path on
failure.

Evidence assets are resolved through the repository registry by stable asset
identifier. Unknown identifiers are errors. Compatibility path remapping may
locate governed evidence under its current domain root, but cannot select a
different asset.

## Synthetic And Evidence Data

Synthetic fixtures test semantics under controlled inputs. Evidence fixtures
represent retained repository observations. A synthetic run must not be
described as release proof, and an evidence file must not be rewritten merely
to simplify a unit test.

The registry records evidence ownership and consumers. Tests should use asset
identifiers where that relationship matters.

## Snapshot Builders

`collect_run_dir_snapshot` captures the governed run layout. Snapshot updates
must be explicit and reviewed. `update_or_assert_snapshot` supports the
repository's update mode but does not decide whether changed output is valid.

Snapshot paths are derived from caller-supplied manifest roots. Generated
snapshots belong in governed fixture locations only when they are intentional
contract assets.

## Builder Review

- Name graph nodes and outputs by their role in the scenario.
- Keep only factors needed by the contract under test.
- Avoid command strings that depend on host-specific tools.
- Make failure and corruption intent explicit.
- Preserve deterministic collection order.
- Add both validity and refusal consumers for boundary fixtures.

## Verification

`fixture_builder_contract.rs`, `fixture_loader_contracts.rs`, and
`evidence_access_contracts.rs` protect construction and lookup. Graph and
artifact consumers prove that fixture semantics still match product behavior.
