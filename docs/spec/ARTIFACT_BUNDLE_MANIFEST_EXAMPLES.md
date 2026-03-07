# Artifact Bundle Manifest Examples

## Minimal

```json
{
  "pack_manifest_version": "artifact-pack/v0.1",
  "artifacts": ["n1:result.json"]
}
```

## Multi-artifact with replay ancestry context

```json
{
  "pack_manifest_version": "artifact-pack/v0.1",
  "artifacts": [
    "extract:raw.csv",
    "transform:clean.csv",
    "train:model.bin"
  ]
}
```

`artifacts` order is canonical and stable for deterministic export diffs.

