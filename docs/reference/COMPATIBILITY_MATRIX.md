# Compatibility Matrix

| surface | current | supported previous | unsupported future handling |
| --- | --- | --- | --- |
| binary CLI | `0.1.x` | patch compatibility | reject unknown flags/commands |
| graph schema | `0.1` | none | fail parse/validation with version diagnostic |
| run-dir format | `run-manifest/v0.1` | none | fail verify/inspect with format diagnostic |
| export bundle | `export-bundle/v0.1` | none | fail import/version-inspect with format diagnostic |
