# Dev Command Surface

## Source of truth
- `src/bijux_cli/cli/commands/dev/__init__.py`
- `src/bijux_cli/cli/commands/dev/atlas/service.py`

## Inventory
- `bijux dev` (callback status payload)
- `bijux dev <tool>` (external binary passthrough to `bijux-dev-<tool>`)
- `bijux dev di`
- `bijux dev list-products`
- `bijux dev list-plugins`

## Notes
- Known tool namespaces: `agent`, `atlas`, `dag`, `dna`, `gnss`, `rag`, `rar`, `vex`.
- Tool passthrough commands forward unknown options to owned product binaries.
