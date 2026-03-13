# Contracts

## Purpose

This section defines the public behavior promises that the repository is willing
to treat as binding. These are not explanations of implementation shape. They
are the promises users, automation, and integrations may rely on.

```mermaid
flowchart TD
    A[Public behavior] --> B[Documented contract]
    B --> C[Test and review evidence]
    C --> D[Compatibility promise]
```

```mermaid
mindmap
  root((Contracts))
    Interface
      command identity
      flags
      exit codes
    Output
      success envelope
      error envelope
      stream routing
    Plugins
      namespace
      lifecycle
      trust and limits
    Distribution
      package naming
      binary ownership
    Change policy
      deprecation
      schema evolution
```

## Read This Set In Order

1. [Interface And Compatibility](interface-and-compatibility.md)
2. [Output And Stream Contracts](output-and-stream-contracts.md)
3. [Plugin Contracts](plugin-contracts.md)
4. [Distribution And Ownership](distribution-and-ownership.md)

## Scope

These pages are intentionally normative:

- they define what is promised, not merely what exists today
- they should be smaller and more stable than guides or architecture docs
- unsupported behavior should be named as unsupported, not implied as stable

## Machine-Readable Contract Assets

These files are part of the kept contract surface and should be reviewed
alongside the markdown pages when a contract changes. The canonical source of
truth lives under `contracts/`. The documentation site publishes copies under
its `contracts/` path:

- `contracts/schemas/output-envelope-v1.schema.json`
- `contracts/schemas/error-envelope-v1.schema.json`
- `contracts/schemas/plugin-manifest-v2.schema.json`
- `contracts/official_product_namespace_registry.json`
- `contracts/product_mount_metadata_contract.json`

## Next Step

If you need implementation shape, go to [Architecture](../04-architecture/index.md).
If you need exact lookup tables, go to [Reference](../06-reference/index.md).
