# Replay Fidelity Levels

Replay fidelity is reported with explicit levels:

- `strict_equivalent`: replay result is semantically equivalent to source run across manifest, graph fingerprint, node outcomes, and output hashes.
- `diverged`: one or more trust dimensions differ and replay proof reports the mismatch reasons.

Implementation anchors:
- `crates/bijux-dag-app/src/replay/diff.rs` (`ReplayEquivalence`, mismatch grouping and reason report)
- `crates/bijux-dag-app/src/lib.rs` (`dag replay --prove` JSON + human-readable proof surface)
- `configs/schema/operator/replay_proof.schema.json` (wire-format contract)

`replay` means re-executing from recorded run evidence and checking fidelity.
`rerun` means a new execution without required equivalence proof.
