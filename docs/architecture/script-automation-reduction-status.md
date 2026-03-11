# Script Automation Reduction Status

## Baseline Freeze
Maintainer automation behavior is now expected to run through `bijux dev cli` commands first.

## Evidence
- `artifacts/status/dev_cli_inventory.json`
- `artifacts/status/script_only_behaviors.json`
- `artifacts/status/make_target_only_behaviors.txt`

## Guardrails
1. New script files under `scripts/` require a justification entry in `.github/script_additions_allowlist.txt`.
2. CI must fail when script additions are missing justification.
3. Contributor and reviewer guidance must prefer `bijux dev cli` over one-off script growth.
