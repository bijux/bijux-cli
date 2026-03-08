# Command taxonomy

Audience: operators and maintainers.
Owner: CLI platform team.
Status: stable.

## User-facing product commands

- `init`
- `validate`
- `canonicalize`
- `lint`
- `fingerprint`
- `show-effective-plan`
- `run`
- `replay`
- `diff`
- `explain`
- `node`
- `status`
- `verify`
- `fsck`
- `hash graph`
- `hash run`
- `hash artifact`
- `capabilities`
- `cache`
- `adapters`
- `export`
- `import`
- `version`
- `migrate dag`
- `migrate run`

## Debug and diagnostics commands

- `doctor`
- `trace-artifact`
- `why-rerun`
- `why-cache-missed`

## Stability note

Commands in this list are those intended for stable user-facing behavior. Commands under maintenance namespaces or service-control work should be governed through explicit compatibility contracts.

## Top-level utility commands

- `completions`

## Canonical spec sources

Normative command contract details are in:

- [CLI backward compatibility contract](../spec/CLI_BACKWARD_COMPATIBILITY.md)
- [CLI deprecation and alias policy](../spec/CLI_DEPRECATION_AND_ALIAS_POLICY.md)
- [CLI ownership boundaries](../spec/CLI_OWNERSHIP.md)
