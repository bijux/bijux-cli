# Configuration Input Inventory

## Scope
Inventory of configuration input sources currently used in repository behavior.

## Input sources
1. CLI flags and subcommand arguments (`bijux`, `bijux-dev-dag`).
2. Explicit JSON config files (`--config`-style and policy fixtures).
3. Environment variables:
   - `BIJUX_DAG_CACHE_DIR`
   - `BIJUX_DAG_ADAPTERS_DIR`
4. In-code defaults from runtime/app config models.

## Ownership
- Runtime execution policy defaults: `bijux-dag-runtime`.
- App-facing config resolution and normalization: `bijux-dag-app`.
- Repo linting and drift checks: `bijux-dev-dag`.

## Enforcement surfaces
- `docs/spec/CONFIG_CONTRACT.md`
- `configs/schema/policy_config.schema.json`
- `configs/schema/runtime_config.schema.json`
