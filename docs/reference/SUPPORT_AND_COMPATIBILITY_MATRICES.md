# Support and compatibility matrices

Audience: operators and maintainers.  
Owner: platform documentation guild.  
Status: stable.

This page is the canonical reference for execution support posture and compatibility surfaces.

## Runtime and backend support

| Surface | Status | Evidence source |
| --- | --- | --- |
| Linux local execution | supported | runtime/app contract suites |
| macOS local execution | supported | runtime/app contract suites |
| Windows local execution | experimental | runtime/app contract suites |
| Local process backend | implemented | runtime execution contracts |
| Container backend | simulated boundary | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| Kubernetes backend | simulated boundary | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| Remote backend | simulated boundary | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| Batch/HPC backend | simulated boundary | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| CLI JSON output contracts | supported | CLI/app output contracts |
| Library crate API stability | internal/experimental | crate boundary contracts |

## Backend capability details

| Surface | Status | Evidence source |
| --- | --- | --- |
| Kubernetes capability report (`dag capabilities --backend kubernetes`) | implemented | CLI/app contract tests |
| Kubernetes execution backend | simulated | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| HPC capability report (`dag capabilities --backend hpc`) | implemented | CLI/app contract tests |
| HPC execution backend | simulated | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| Remote capability report (`dag capabilities --backend remote`) | implemented | CLI/app contract tests |
| Remote execution backend | simulated | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |

## Compatibility matrix

| Surface | Current | Supported previous | Unsupported future handling |
| --- | --- | --- | --- |
| binary CLI | `0.1.x` | patch compatibility | reject unknown flags/commands |
| graph schema | `0.1` | none | fail parse/validation with version diagnostic |
| run-dir format | `run-manifest/v0.1` | none | fail verify/inspect with format diagnostic |
| export bundle | `export-bundle/v0.1` | none | fail import/version-inspect with format diagnostic |
| artifact index | `outputs-index/v0.1` | none | reject with format diagnostic |
| proof bundle | `proof-bundle/v0.1` | none | reject with schema-version diagnostic |

## Ecosystem compatibility posture

| Component | Compatibility surface | Current policy |
| --- | --- | --- |
| bijux-cli | command composition contract | must preserve `bijux-dag` semantics |
| bijux-dag | identity/replay/artifact contracts | source of truth |
| bijux-atlas | adapter capability consumption | extend-only |
| bijux-dna | HPC capability consumption | extend-only |
