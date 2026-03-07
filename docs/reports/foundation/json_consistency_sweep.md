# JSON Consistency Sweep

Operator commands with JSON envelope support include:

- `run`, `replay`, `diff`, `why-rerun`, `why-cache-missed`
- `runs inspect`, `runs history`, `runs id-explain`
- `trace-artifact`, `artifact-inspect`

Consistency rule:

- envelope fields: `ok`, `status`, `command`, `data`, `diagnostics`, `error`

