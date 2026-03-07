# Adoption Surfaces

## Official product surfaces
- CLI command tree (`dag ...`)
- run-directory artifacts
- export bundles
- config and schema files

## Crate stability levels
- internal:
  - `bijux-dag-runtime`
  - `bijux-dag-core`
  - `bijux-dag-artifacts`
- experimental:
  - public crate use outside the CLI binary
- supported:
  - CLI interface and documented JSON contracts

## External consumption policy
- Rust crates are internal-first unless explicitly documented as stable APIs.
- Quickstarts and installation docs must reference only supported surfaces.

## Machine-readable capability surface
- `dag capabilities --json` is the canonical summary of currently supported and simulated surfaces.
