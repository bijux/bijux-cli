# CLI

Bijux provides an umbrella CLI `bijux` with sub-apps. The DAG app is available as `bijux dag`.

## Commands

```
bijux dag validate <dag>
bijux dag run <dag> --out <runs/>
bijux dag replay <run-dir> --out <runs/>
bijux dag diff <runA> <runB>
bijux dag explain <run-dir> [--node <id>]
bijux dag node <run-dir> --id <id>
bijux dag status <run-dir>
bijux dag verify <run-dir>
bijux dag cache <ls|pack|unpack|verify|gc>
bijux dag adapters <ls|doctor>
bijux dag export <run-dir> --out bundle.json
bijux dag import <bundle.json>
```

## JSON Envelope

All commands accept a global `--json` flag. JSON output is normalized as:

```
{
  "ok": true,
  "command": "dag.validate",
  "data": { ... },
  "diagnostics": [ ... ]
}
```

`diagnostics` is used for validation/lint warnings or errors. Other commands return an empty array.

## Deprecation Note

The legacy `bijux-dag` binary remains as a thin wrapper and will print a deprecation warning.
Use `bijux dag ...` going forward.
