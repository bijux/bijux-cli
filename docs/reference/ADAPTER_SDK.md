# Adapter SDK contracts

`bijux-dag-runtime` exposes typed plugin contracts for adapter and backend integrations.

## Adapter plugin contract

- Adapter identity: name and version
- Capability declaration
- Typed adapter execution context
- Typed node result envelope
- Plugin trust and isolation policy requirements

## Backend plugin contract

- Backend kind declaration
- Typed execution request submission
- Typed completion polling contract
- Executor capability compatibility declaration

## Additional extension boundaries

Stable plugin boundaries also include:

- artifact store plugins
- observability sink/exporter plugins

These boundaries use the same metadata, compatibility, and conformance model.

## Manifest contract

Plugins publish a manifest with:

- plugin name
- plugin version
- plugin type
- contract version

See:

- [Extension catalog contracts](./EXTENSION_CATALOG_CONTRACTS.md)
- [Plugin and DSL roadmap](./PLUGIN_DSL_ROADMAP.md)
