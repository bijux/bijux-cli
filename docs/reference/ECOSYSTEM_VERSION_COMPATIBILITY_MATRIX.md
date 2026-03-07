# Ecosystem Version Compatibility Matrix

| component | compatibility surface | current policy |
| --- | --- | --- |
| bijux-cli | command composition contract | must preserve `bijux-dag` semantics |
| bijux-dag | identity/replay/artifact contracts | source of truth |
| bijux-atlas | adapter capability consumption | extend-only |
| bijux-dna | HPC capability consumption | extend-only |
