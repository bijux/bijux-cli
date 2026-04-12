# Maintainer Packages

This section is the package ownership map for maintainer and control-plane behavior. Use it when the question is about repository automation, diagnostics, release verification, or governance enforcement rather than product runtime behavior.

## Package Map

| Package | Owns | Enter Here When |
| --- | --- | --- |
| [`bijux-dev`](bijux-dev.md) | Repository control plane, maintainer automation, diagnostic/reporting flows, and release-verification surfaces | the issue is repository health, quality gates, release evidence, or governance tooling behavior |

## Navigation Rule

Use this package section for repository-level operations and policy execution logic. If the issue is product behavior (`bijux` runtime or DAG semantics), switch to the CLI or DAG handbook package sections instead of extending maintainer scope.
