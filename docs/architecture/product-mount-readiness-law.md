# Product Mount Readiness Law

## Scope

This law keeps official product mount readiness small and explicit.

## Canonical sources

- Namespace registry: `docs/constitution/official_product_namespace_registry.json`
- Metadata contract: `docs/constitution/product_mount_metadata_contract.json`
- Rust contract export: `KNOWN_BIJUX_TOOLS`, `OFFICIAL_PRODUCT_NAMESPACES`, and `ProductMountMetadata`

## Commitments

- Official namespaces are reserved for product mounts and cannot be claimed by plugins.
- Routing and plugin validation consume the same reserved namespace contract.
- Readiness is provided by metadata, tests, and reports.

## Non-commitments

- No dynamic product loading system.
- No external ABI promise for product mounts.
- No network registry promise for product discovery.

## Maintenance rules

- Keep `KNOWN_BIJUX_TOOLS` and `official_product_namespace_registry.json` in exact sync.
- Add entries only for repositories accepted under the Bijux organization tool contract.
- Keep plugin/runtime law stable and minimal.
- Reject speculative runtime complexity.
