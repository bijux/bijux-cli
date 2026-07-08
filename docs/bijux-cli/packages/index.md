# CLI Packages

Use this page when the CLI surface is clear but the owning package is not.

`bijux-cli` holds native command behavior. `bijux-cli-python` carries the
Python distribution surface and bridge back into the same runtime contract.

## Package Map

| Package | Owns | Enter Here When |
| --- | --- | --- |
| [`bijux-cli`](bijux-cli.md) | Native runtime semantics, command routing, executable behavior, and contract-facing CLI surfaces | the issue is flags, output shape, exit behavior, routing, or runtime execution semantics |
| [`bijux-cli-python`](bijux-cli-python.md) | Python distribution surface, launcher bridge behavior, packaging metadata, and cross-language runtime parity | the issue is Python install/entrypoint behavior, bridge compatibility, or release packaging alignment |

## Package Guides

- [Python Bridge Guide](python-bridge-guide.md)

## Reading Rule

Start here when ownership is unclear. Move to the package page only after
deciding whether the change belongs to runtime command semantics
(`bijux-cli`) or Python distribution and bridge semantics
(`bijux-cli-python`).

If a change spans both packages, treat it as a cross-package contract change
and validate both package pages before implementation.
