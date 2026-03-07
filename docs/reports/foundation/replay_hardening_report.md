# Replay hardening report

## Scope

Captures replay semantics, reason reporting, schema coverage, and governance evidence.

## Replay proof boundary

Replay proves semantic equivalence or mismatch only from authoritative artifacts:

- `manifest.json`
- graph snapshot and fingerprint
- node outcomes and traces
- outputs index and output hashes
- provenance markers and replay source metadata

Replay does not prove uncaptured external side effects.

## Semantic diff and explainability

Required replay surfaces:

- `dag diff --mode semantic`
- `dag runs diff --mode semantic`
- explain mode grouped by mismatch cause classes

Replay reason report includes:

- equivalence decision
- mismatch reasons
- compared dimensions
- mismatch dimensions
- grouped cause counts

## Fixture and schema coverage

Mandatory replay fixture family:

- match
- mismatch
- corruption
- unsupported version

Machine-readable replay schema:

- `configs/schema/operator/replay_diff.schema.json`

## Battle trust linkage

Replay hardening protects trust property `tp_replay_equivalence` and remains mandatory battle evidence.
