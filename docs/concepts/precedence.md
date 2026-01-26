# Precedence

Bijux CLI resolves configuration in a strict order. Higher layers override lower
layers.

Order (highest to lowest):

1. CLI flags
2. Environment variables
3. Config file
4. Defaults

Rules:

- Explicit inputs always win
- Defaults never override explicit values
- Unrelated outputs must not change ordering

Example

```
config file: format=yaml
env: BIJUXCLI_FORMAT=json
cli: --format json
result: json
```

See reference/config_sources.md for the full matrix.
