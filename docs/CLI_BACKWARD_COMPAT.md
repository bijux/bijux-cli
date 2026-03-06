# CLI backward-compat policy

## Compatibility surface

The following command contracts are treated as stable:
- `dag` subcommands (`validate`, `run`, `replay`, `diff`, `explain`, `cache`, `adapters`, `status`, `node`, `verify`).
- `--json` output shape for command contracts.
- Stable exit-code classes: success and validation/runtime failure.

## Placeholder commands

`rag` and `rar` remain reserved placeholders and return a documented stable failure code until a contract is approved.

## Change process

- New top-level commands require an accompanying fixture or migration plan.
- Removal of stable commands requires explicit deprecation path and changelog entry.
