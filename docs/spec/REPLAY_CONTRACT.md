# Replay Contract

## Scope

This contract defines what `bijux-dag` replay is allowed to claim, which
inputs are authoritative for replay comparison, and which proof boundaries must
stay stable across CLI, runtime, evidence fixtures, and repository governance.

Replay exists to answer a narrow question: whether a later execution can be
compared against a recorded run with deterministic, reviewable semantics.

## Replay definition

Replay compares a candidate run against authoritative run evidence and classifies
the result using stable replay and diff vocabulary.

Replay is not a generic "looks similar" feature. It is a contract-governed
comparison that depends on recorded graph identity, run material, artifact
availability, and explicit mismatch reasons.

## Authoritative inputs

The following surfaces are authoritative for replay behavior:

- `configs/dag/schema/operator/replay_diff.schema.json`
- `crates/bijux-dag-app/src/commands/mod.rs`
- `crates/bijux-dag-app/tests/replay_contract.rs`
- `crates/bijux-dag-runtime/tests/replay_contract.rs`
- `crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs`
- `evidence/cache/replay/regression_corpus.json`
- `evidence/battle/workflows/replay/replay_semantic_comparison.json`

Replay fixtures under `evidence/cache/replay/` are the canonical evidence set
for match, mismatch, corruption, unsupported-version, cache-hit, cache-miss,
and missing-artifact scenarios.

## Replay explain mode

Replay explain mode must provide deterministic reason categories for success,
mismatch, and validation failure paths. It must stay aligned with semantic diff
mode in CLI surfaces and with the replay regression corpus that backs those
reason codes.

Operator-facing docs may describe a surface as replayable only when they cite `docs/spec/REPLAY_CONTRACT.md` directly.

## Replay bundle boundary

The portable replay bundle surface is the export bundle governed by
`docs/spec/IMPORT_EXPORT_CONTRACT.md`, not the diagnostics bundle emitted by
`runs diagnostics-bundle`.

The current bundle boundary is:

- `export-bundle/v0.1` with `--with-files` is the artifact-bearing replay bundle
  mode
- `manifest-only` and `without-artifacts` export bundles preserve structural
  evidence and provenance, but they do not carry the full file payload required
  for artifact-backed replay proof
- diagnostics bundles exist for inspection and support capture; they are not an importable replay contract and must not be treated as replay bundles

## Node rerun boundary

When replay is scoped with `--from-node`, the selected downstream closure
becomes a rerun boundary instead of a generic selector convenience.

The rerun-boundary contract is:

- source runs may be addressed by run directory or by `--source-run-id` plus a
  replay source root
- the boundary node is selected by exact node id and expands to a deterministic
  downstream closure
- persisted inputs crossing into that boundary must be verified against the
  source run's node output indexes, node fingerprints, and artifact hashes
- replay must refuse execution when the persisted boundary evidence is missing,
  unreadable, or hash-inconsistent
- when the rerun boundary contains exactly one selected root, replay must
  surface a focused node diff that explains what changed for that rerun target

## What replay cannot prove

Replay does not prove business correctness, intent correctness, or external side
effects outside the recorded artifact and runtime boundary. Replay also cannot
invent missing evidence. When required artifacts, manifests, or compatible
runtime evidence are absent, replay must fail explicitly instead of silently
downgrading trust.

## Related tests

- `crates/bijux-dag-app/tests/replay_contract.rs`
- `crates/bijux-dag-runtime/tests/replay_contract.rs`
- `crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs`
- `crates/bijux-dev/tests/replay_hardening_contracts.rs`
- `crates/bijux-dev/tests/replay_mismatch_corpus_contracts.rs`

## Versioning and change policy

Replay classification vocabulary, authoritative inputs, and proof limitations
are stable contract surfaces. Any incompatible change requires updating this
document, refreshing the replay hardening report, and extending the linked
contract tests in the same change.
