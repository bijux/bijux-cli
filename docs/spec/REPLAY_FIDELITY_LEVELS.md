# Replay Fidelity Levels

Replay fidelity is reported with explicit levels:

- `strict_equivalent`: replay result is semantically equivalent to source run across manifest, graph fingerprint, node outcomes, and output hashes.
- `diverged`: one or more trust dimensions differ and replay proof reports the mismatch reasons.

`replay` means re-executing from recorded run evidence and checking fidelity.
`rerun` means a new execution without required equivalence proof.
