# CLI Packages

This section is the package ownership map for the CLI handbook. Use it when a question is about command behavior, route dispatch, binary/runtime parity, or Python distribution boundaries and you need to land in the exact package contract quickly.

## Package Map

| Package | Owns | Enter Here When |
| --- | --- | --- |
| [`bijux-cli`](bijux-cli.md) | Native runtime semantics, command routing, executable behavior, and contract-facing CLI surfaces | the issue is flags, output shape, exit behavior, routing, or runtime execution semantics |
| [`bijux-cli-python`](bijux-cli-python.md) | Python distribution surface, launcher bridge behavior, packaging metadata, and cross-language runtime parity | the issue is Python install/entrypoint behavior, bridge compatibility, or release packaging alignment |

## Navigation Rule

Start from this page when ownership is unclear. Move to the package page only after deciding whether the change belongs to runtime command semantics (`bijux-cli`) or Python distribution/bridge semantics (`bijux-cli-python`).

If a change spans both packages, treat that as a cross-package contract change and validate both package pages before implementation.
