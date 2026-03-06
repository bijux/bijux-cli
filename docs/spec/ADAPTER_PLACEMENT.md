# Adapter placement and boundary

## Decision
Built-in adapters remain in `bijux-dag-runtime` for now.

## Required boundary
- Runtime must expose adapter contracts through `runtime::adapter_api`.
- Adapter-specific implementation details must stay in adapter-focused modules.
- Cross-module execution logic must consume adapter contracts, not adapter-specific concrete details.

## Future option
A dedicated `bijux-dag-adapters` crate remains a valid future extraction once adapter contracts and runtime layering are fully stable.
