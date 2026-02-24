# Atlas Local Routing

## Goal
Run Atlas runtime and Atlas control-plane commands through the umbrella CLI.

## Prerequisites
- `bijux-cli` repository checked out and runnable.
- `bijux-atlas` repository checked out and binaries built.

## Build Product Binaries
From the atlas repository:

```bash
make install-local
```

This installs both required binaries into `artifacts/bin`:
- `bijux-atlas`
- `bijux-dev-atlas`

## Configure Discovery
In your shell:

```bash
export BIJUX_DEV_MODE=1
export BIJUXCLI_PRODUCT_BIN_DIR="/path/to/bijux-atlas/artifacts/bin"
```

## Run Through Umbrella
Runtime surface:

```bash
bijux atlas atlas --help
```

Control-plane surface:

```bash
bijux dev atlas --help
```

## Verify Discovery

```bash
bijux dev list-products --format json
```

The output should list resolved paths for both binaries under `products.atlas`.
