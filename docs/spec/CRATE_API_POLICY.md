# Crate API policy

## Public visibility defaults
- Default visibility is `pub(crate)`.
- `pub` is allowed only for documented crate contracts.
- New public exports require corresponding crate documentation updates.

## CLI boundary
- `bijux-dag-cli` is a thin binary crate only.
- Business logic is forbidden in `bijux-dag-cli`.

## Core boundary
`bijux-dag-core` exports stable model, parse, resolve, validation, topology, and fingerprint API surfaces.
Core exports must remain deterministic and side-effect free.

## Artifact boundary
`bijux-dag-artifacts` is an artifact model and persistence API crate.
- It may own artifact storage operations through stable APIs.
- Runtime may interact with artifact persistence only through public artifact APIs.

## Formatting helper reuse
No crate may depend on another crate solely to reuse rendering or formatting helpers.
Shared rendering helpers must live in the consuming crate or a dedicated neutral utility crate.

## Adapter placement
Built-in adapters remain in `bijux-dag-runtime` for now.
- Adapter contracts and type-level boundaries are exposed through `runtime::adapter_api`.
- Adapter-specific execution logic must not leak across unrelated runtime modules.
