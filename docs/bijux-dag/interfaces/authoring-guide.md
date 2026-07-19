---
title: Authoring Guide
audience: developers
type: guide
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Authoring Guide

A DAG definition is an execution contract, not a drawing. Author it so a
reviewer can identify graph inputs, effects, outputs, failure policy, selection
behavior, and retained evidence before any node runs.

The [Graph Schema Reference](graph-schema.md) owns field-level syntax. This
guide owns the authoring and review workflow. The machine schema remains
`configs/dag/schema/dag.schema.json`.

## Choose A Governed Starting Point

Repository assets have different authority:

| Asset family | Meaning | Use |
| --- | --- | --- |
| `evidence/dag/authoring/patterns/` | normative topology and authoring shapes | start a graph structure or test a graph rule |
| `evidence/dag/authoring/examples/` | illustrative executable workflows | study complete input, effect, and output choices |
| `evidence/dag/authoring/negative/` | normative refusal cases | confirm that an invalid move fails for the governed reason |

Start with the smallest pattern that expresses the dependency shape. Use an
example when the workflow needs a concrete contract already exercised by the
repository:

| Need | Starting asset |
| --- | --- |
| one executable node | `patterns/minimal.json` |
| chain, fan-out, or aggregation | the matching `patterns/pattern_*.json` |
| typed file-processing inputs | `examples/file-processing-report.dag.json` |
| branch decisions and joins | `examples/audience-branch-bulletin.dag.json` |
| retries and approval recovery | `examples/compliance-gated-bulletin.dag.json` |
| container execution | `examples/release-note-bundle.dag.json` |

Scheduled and backfill examples are repository-owned internal lanes in
`v0.4.0`. Their presence does not make scheduler commands part of the stable
operator surface.

## Define The Contract Before Commands

Review the graph in this order:

1. Give `meta.name`, node IDs, edge IDs, and output names durable domain
   identities. These names appear in selectors, paths, traces, and retained
   evidence.
2. Declare graph inputs with types, requirements, defaults, and allowed values.
   Do not hide operator inputs in shell text or ambient environment variables.
3. Declare each node input and output. Output paths must be normalized,
   relative, and owned by that node.
4. Bind params through literal values or explicit references. A reference must
   identify its graph input, node output, or path variable source.
5. Declare filesystem, network, environment, and clock effects. Environment
   access also requires an exact allowlist.
6. Set timeout, retry, resource, and cache policy deliberately. Cache opt-out
   requires a reason.
7. Define branch decisions, conditional edges, and join trigger rules as graph
   semantics rather than command-side conventions.

Validation can prove that these declarations are coherent. It cannot prove
that a shell command is safe, that an external service is stable, or that an
output is scientifically or operationally correct.

## Validate Structure And Semantics

From repository root:

```bash
GRAPH="evidence/dag/authoring/examples/file-processing-report.dag.json"

cargo run -p bijux-dag-cli --bin bijux-dag -- validate "${GRAPH}"
```

Strict parsing rejects unknown fields and malformed JSON shapes. Semantic
validation then checks references, topology, paths, effects, engines, cache
policy, branches, and trigger rules. Read the diagnostic `code`, `path`, and
`hint` together; correcting only the message text can leave the same contract
violation elsewhere.

Do not weaken a graph or validator to make an invalid fixture pass. The files
under `evidence/dag/authoring/negative/` intentionally preserve refusal
behavior for cycles, invalid references and selectors, undeclared outputs,
missing required bindings, invalid container workdirs, and unsupported adapter
payloads.

## Review The Lowered Plan

Validation answers whether the graph is accepted. Planning answers what the
runtime intends to execute.

```bash
RUNS_ROOT="./artifacts/authoring-review"

cargo run -p bijux-dag-cli --bin bijux-dag -- plan explain "${GRAPH}" \
  --out "${RUNS_ROOT}" \
  --run-id authoring-review

cargo run -p bijux-dag-cli --bin bijux-dag -- plan closure "${GRAPH}" \
  --select id:render_report
```

Review:

- deterministic node order and edge dependencies;
- resolved adapter kinds and capability requirements;
- effective inputs, outputs, retries, resources, and cache behavior;
- branch and trigger semantics;
- selector closure, including every prerequisite pulled into a focused run;
- planner diagnostics and identity.

A selector is not an authorization boundary. It narrows planned work while
retaining required dependency closure.

## Execute Into A Retained Boundary

Use explicit inputs and keep runs under `artifacts/` during development:

```bash
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"

cargo run -p bijux-dag-cli --bin bijux-dag -- run --json "${GRAPH}" \
  --out "${RUNS_ROOT}" \
  --run-id authoring-review-run \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Authoring Review"
```

After execution, inspect the finalized run rather than relying on the terminal
summary:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- explain \
  "${RUNS_ROOT}/run-authoring-review-run"

cargo run -p bijux-dag-cli --bin bijux-dag -- verify --json \
  "${RUNS_ROOT}/run-authoring-review-run" \
  --strict
```

The retained manifest, node traces, indexes, and payload digests establish what
ran and what survived. They do not establish domain correctness; reviewers
must still inspect the produced content against workflow requirements.

## Change Review

| Graph change | Required review |
| --- | --- |
| input type, default, or required flag | callers, persisted invocations, graph identity |
| node or output rename | selectors, references, retained paths, replay consumers |
| command, image, or adapter change | effects, capabilities, execution identity, security |
| edge, branch, or trigger change | closure, skipped lanes, join behavior |
| retry, timeout, or resources change | failure semantics, capacity, runtime cost |
| cache policy change | identity inputs, side effects, reuse and refusal evidence |
| output contract change | indexes, downstream references, promotion, compatibility |

Keep a graph change and the tests that prove its accepted and rejected behavior
in the same review. When adding a reusable pattern, update
`evidence/dag/authoring/metadata.json`, the authoring contract, and the
consumer tests together.

## Completion Standard

An authored graph is ready for review when:

- strict validation passes without error diagnostics;
- plan explanation matches the intended work and capabilities;
- selector closure is understood for every supported focused run;
- all inputs, outputs, effects, retries, resources, and cache rules are
  explicit;
- one retained run can be inspected and strictly verified;
- domain outputs have been reviewed separately from artifact integrity;
- invalid neighboring cases still fail with the intended diagnostics.

Continue with [Run Evidence Layout](run-evidence-layout.md) for evidence
authority and [Execution Security And Isolation](../operations/security-isolation-truth.md)
for the boundary around executed code.
