# Workflows And Services

Application workflows coordinate domain owners around an operator goal. They
should expose inputs, decisions, outputs, and recovery without duplicating the
algorithms they invoke.

## Graph Workflows

Validate, lint, canonicalize, fingerprint, inspect, and plan routes load graph
sources through `read`, then delegate parsing, validation, identity, and
planning to `bijux-dag-core`. Multiple source graphs use core composition
rules.

The app may select strictness, request explanations, or shape diagnostics. It
cannot weaken a core refusal or compute an alternate fingerprint.

## Run Workflows

Run routes resolve effective configuration, backend selection, paths, and
operator policy before constructing runtime input. `bijux-dag-runtime` owns
admission, scheduling, attempts, cache, and execution. The app reports run
identity and retained evidence locations.

Progress, stop, status, history, tree, timeline, and failure routes inspect
runtime/artifact evidence. They do not infer a better terminal state than the
retained records support.

## Evidence Workflows

Inspect, verify, prove, import, export, hash, and repair routes call
`bijux-dag-artifacts` or runtime evidence services. Verification failure,
corruption, unsupported schema, and unsafe path remain explicit.

Repair first produces a reviewable proposal. Applying repair preserves source
evidence and records changed facts and lineage.

## Cache And Replay

Cache routes explain keys, verify entries, compare state, pack/unpack portable
entries, and simulate pruning. Replay routes inspect source eligibility,
construct a replay plan, execute allowed work through runtime, and report
ancestry and proof.

The app never converts a cache miss to a hit or a replay refusal to replay
success for presentation convenience.

## Configuration

`RuntimeSurfaceConfig` and `PolicySurfaceConfig` represent command-facing
configuration. Resolution combines defaults, files/profiles, environment, and
explicit values while preserving precedence. Normalization happens once;
`config_fingerprint` records execution-relevant effective values.

Secrets and display-only values do not enter the fingerprint.

## Service Design

A reusable application service:

- accepts explicit values and paths;
- returns typed data or a classified error;
- does not print directly;
- does not parse Clap matches;
- calls one domain authority for domain behavior;
- is testable with isolated fixtures.

Routes adapt command models to services. Renderers adapt responses to streams.

## Verification

Workflow contracts are grouped by domain: graph/plan, run/history/progress,
cache, replay/diff, import/export, repair, container, Kubernetes, and SLURM.
Use the focused contract matching the changed workflow and its error/output
contract.
