# Dev Command Surface

## Source of truth
- `src/bijux_cli/cli/commands/dev/__init__.py`
- `src/bijux_cli/cli/commands/dev/atlas/service.py`

## Inventory
- `bijux dev` (callback status payload)
- `bijux dev atlas` (external binary passthrough to `bijux-dev-atlas`)
- `bijux dev di`
- `bijux dev list-products`
- `bijux dev list-plugins`

## Notes
- `dev atlas` is a command group with passthrough behavior and unknown-option forwarding.
