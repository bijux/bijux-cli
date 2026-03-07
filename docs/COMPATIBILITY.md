# Compatibility guarantees

## Spec compatibility

- Current spec version: `bijux-dag/v0.1`.
- Unknown spec values must fail strict parsing for invalid compatibility surfaces.
- Canonical JSON and fingerprint contracts must remain stable for unchanged graphs.

## Runtime compatibility

- Replay and golden contracts compare graph fingerprints and manifest shape.
- Diff contracts are expected to be deterministic for repeated identical runs.

## CLI compatibility

- `dag validate`, `run`, `replay`, `diff`, `status`, `cache`, and `adapters` are contract surfaces.
- `dag hash run`, `dag hash artifact`, and `dag fsck` are stable compatibility surfaces.
