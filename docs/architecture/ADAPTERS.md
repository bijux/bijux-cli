# Adapters

Adapters are deterministic execution backends. Each adapter exposes:
- `adapter_id`
- `adapter_version`
- required effects
- lifecycle: `prepare`, `execute`, `cleanup`

## Rules
- Deterministic outputs for identical inputs.
- No hidden effects: all effects must be declared.
- Stable versioning when behavior changes.

## Adding a New Adapter
1. Define a new adapter struct implementing `Adapter`.
2. Specify required effects.
3. Update `registered_adapters()`.
4. Add tests to cover determinism and outputs.
