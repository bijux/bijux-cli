# Adapter SDK contracts

`bijux-dag-runtime` exposes typed plugin contracts for adapter and backend integrations.

## Adapter plugin contract

- Adapter identity: name and version
- Capability declaration
- Typed adapter execution context
- Typed node result envelope

## Backend plugin contract

- Backend kind declaration
- Typed execution request submission
- Typed completion polling contract

## Manifest contract

Plugins publish a manifest with:

- plugin name
- plugin version
- plugin type
- contract version
