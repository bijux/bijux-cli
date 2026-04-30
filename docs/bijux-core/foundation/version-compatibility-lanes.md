# Version Compatibility Lanes

Source contract: `contracts/foundation/version_compatibility_lanes.v1.json`.

This surface codifies compatibility lanes for Level-1 foundations:

- CLI envelopes
- mount descriptors
- graph specs
- run manifests
- artifact indexes
- replay bundles

Lane semantics:

- `current`: production version for normal writes and reads.
- `previous`: explicitly accepted legacy version for compatibility reads.
- `refused`: must be rejected with stable diagnostics.

Enforcement:

- `bijux-cli` exposes `version_compatibility_lanes_query()` as typed query output.
- `bijux-dev` validates contract/query alignment and fixture lane classification.
- `bijux-dev` validates graph spec current/previous/refused behavior against `parse_graph_strict`.
