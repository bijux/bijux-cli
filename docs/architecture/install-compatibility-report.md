# Install Compatibility Report

Date: 2026-03-09

## Coverage status

Implemented and tested:

- Linux-style path resolution
- macOS-style path resolution
- Windows-style path resolution
- HOME override behavior
- XDG-like home path behavior
- Config/history/plugins default path compatibility with Python expectations (`~/.bijux/.env`, `~/.bijux/.history`, `~/.bijux/.plugins`)
- Duplicate install detection diagnostics
- PATH shadowing diagnostics
- Completion script generation and completion target path generation
- Shell detection helper behavior
- `cli doctor` integration assertions for install diagnostics fields
- `cli paths` integration assertions for install diagnostics fields and post-install hint
- Compatibility notes for pip and cargo users
- Sanity performance check for install health diagnostics path

## Known remaining gaps

1. Dedicated platform CI matrix for filesystem permission edge-cases is not yet configured here.
2. Completion-path installation side effects are validated at helper level, not full installer execution workflow.
3. More exhaustive large-path-list stress tests can be added if path discovery becomes a hotspot.
