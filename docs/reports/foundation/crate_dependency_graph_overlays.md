# Crate Dependency Graph Overlays

generated_from: `docs/reports/foundation/crate_graph_snapshot.json`

## Kernel Overlay

- `bijux-dag-core`
- `bijux-dag-artifacts`
- `bijux-dag-runtime`

## App Overlay

- `bijux-dag-app`
- `bijux-dag-cli`

## Governance Overlay

- `bijux-dev-dag`

## Workspace Edges

- `bijux-dag-app -> bijux-dag-core`
- `bijux-dag-app -> bijux-dag-runtime`
- `bijux-dag-app -> bijux-dag-artifacts`
- `bijux-dag-cli -> bijux-dag-app`
- `bijux-dev-dag -> bijux-dag-core`
- `bijux-dev-dag -> bijux-dag-runtime`
