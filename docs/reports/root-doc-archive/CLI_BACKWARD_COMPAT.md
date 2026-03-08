# CLI backward-compat policy

## Compatibility surface

The following command contracts are treated as stable:
- `dag` subcommands (`validate`, `run`, `replay`, `diff`, `explain`, `cache`, `adapters`, `status`, `node`, `verify`).
- `dag` hash and diagnostics surfaces (`hash graph`, `hash run`, `hash artifact`, `fsck`, `capabilities`).
- `--json` output shape for command contracts.
- Stable exit-code classes: success and validation/runtime failure.

## Change process

- New top-level commands require an accompanying fixture or migration plan.
- `dag fsck` is a stable alias surface for run-directory verification.
- Removal of stable commands requires explicit deprecation path and changelog entry.

See `docs/spec/CLI_DEPRECATION_AND_ALIAS_POLICY.md` for alias and deprecation requirements.
